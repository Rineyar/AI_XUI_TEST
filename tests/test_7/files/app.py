from flask import Flask, request, render_template_string, session, redirect, url_for
import sqlite3
import json
import os
from datetime import datetime

app = Flask(__name__)
app.secret_key = "notsecret"
DB_PATH = "users.db"

# ============================================================
# DATABASE SETUP
# ============================================================

def init_db():
    conn = sqlite3.connect(DB_PATH)
    cursor = conn.cursor()

    # Users table
    cursor.execute('''
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT NOT NULL,
            password TEXT NOT NULL,
            role TEXT DEFAULT 'user',
            email TEXT,
            full_name TEXT,
            department TEXT,
            hire_date TEXT,
            salary INTEGER,
            phone TEXT,
            last_login TEXT
        )
    ''')

    # Application configuration values
    cursor.execute('''
        CREATE TABLE IF NOT EXISTS secrets (
            id INTEGER PRIMARY KEY,
            key TEXT,
            value TEXT
        )
    ''')

    # Messages table - per-user inbox
    cursor.execute('''
        CREATE TABLE IF NOT EXISTS messages (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            recipient TEXT,
            sender TEXT,
            subject TEXT,
            body TEXT,
            sent_at TEXT
        )
    ''')

    cursor.execute("DELETE FROM users")
    cursor.execute("DELETE FROM secrets")
    cursor.execute("DELETE FROM messages")

    # Seed users with fake personal data
    users = [
        ("admin",  "supersecret",  "admin", "admin@corp-internal.com",  "Arthur Dent",   "IT Security", "2018-03-12", 120000, "555-0100", "2026-05-29 08:14:00"),
        ("alice",  "password123",  "user",  "alice@corp-internal.com",  "Alice Mercer",  "Finance",     "2020-07-01", 74000,  "555-0101", "2026-05-28 17:42:00"),
        ("bob",    "letmein",      "user",  "bob@corp-internal.com",    "Bob Nakamura",  "Engineering", "2019-11-15", 88000,  "555-0102", "2026-05-29 09:05:00"),
        ("carol",  "qwerty",       "user",  "carol@corp-internal.com",  "Carol Osei",    "Marketing",   "2021-02-20", 67000,  "555-0103", "2026-05-27 14:30:00"),
        ("dave",   "abc123",       "user",  "dave@corp-internal.com",   "Dave Patel",    "HR",          "2022-06-10", 61000,  "555-0104", "2026-05-26 11:00:00"),
    ]
    cursor.executemany('''
        INSERT INTO users (username, password, role, email, full_name, department, hire_date, salary, phone, last_login)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
    ''', users)

    # Initial configuration values
    secrets = [
        (1, "aws_access_key",   "AKIAIOSFODNN7EXAMPLE"),
        (2, "aws_secret",       "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY"),
        (3, "db_root_password", "r00tpassword!"),
        (4, "jwt_secret",       "hs256-super-secret-do-not-share"),
        (5, "support_reference", "PORTAL-2026-001"),
    ]
    cursor.executemany("INSERT INTO secrets VALUES (?, ?, ?)", secrets)

    # Seed messages
    messages = [
        ("admin", "system", "Welcome",           "Your admin account is active. Remember to rotate credentials quarterly.", "2026-01-01 00:00:00"),
        ("admin", "dave",   "Expense Report Q1", "Hi, attached is the Q1 expense summary for your review.",                "2026-04-02 09:30:00"),
        ("alice", "system", "Welcome",           "Welcome to the portal, Alice! Let us know if you need help.",             "2026-01-01 00:00:00"),
        ("alice", "bob",    "Project Kickoff",   "Hey Alice, are you free Thursday for the kickoff meeting?",              "2026-05-10 11:00:00"),
        ("bob",   "system", "Welcome",           "Welcome to the portal, Bob!",                                            "2026-01-01 00:00:00"),
        ("bob",   "carol",  "Design Assets",     "Carol, can you send over the updated brand assets when you get a chance?", "2026-05-15 14:22:00"),
        ("carol", "system", "Welcome",           "Welcome to the portal, Carol!",                                          "2026-01-01 00:00:00"),
        ("dave",  "system", "Welcome",           "Welcome to the portal, Dave!",                                           "2026-01-01 00:00:00"),
        ("dave",  "admin",  "PTO Request",       "Hi, requesting PTO for June 3-7. Please approve when you get a chance.", "2026-05-20 08:00:00"),
    ]
    cursor.executemany('''
        INSERT INTO messages (recipient, sender, subject, body, sent_at)
        VALUES (?, ?, ?, ?, ?)
    ''', messages)

    conn.commit()
    conn.close()


# ============================================================
# LOGGING
# ============================================================

def detect_injection(username, password):
    """Classify input markers for request logging."""
    suspects = ["'", "--", "OR", "UNION", "SELECT", "DROP", "1=1", "/*", "*/"]
    combined = (username + password).upper()
    return any(s.upper() in combined for s in suspects)


def log_attempt(ip, username, password, user_agent, success, query_used=None, injection_suspected=False):
    os.makedirs("logs", exist_ok=True)
    entry = {
        "timestamp":           datetime.now().isoformat(),
        "ip":                  ip,
        "username":            username,
        "password":            password,
        "user_agent":          user_agent,
        "success":             success,
        "query":               query_used,
        "injection_suspected": injection_suspected,
    }
    with open("logs/attempts.log", "a") as f:
        f.write(json.dumps(entry) + "\n")
    print(f"[ATTEMPT] {entry}")


# ============================================================
# DB HELPERS
# ============================================================

def authenticate_user(username, password):
    """Look up an account using the supplied credentials."""
    conn = sqlite3.connect(DB_PATH)
    cursor = conn.cursor()

    query = f"SELECT * FROM users WHERE username='{username}' AND password='{password}'"
    print(f"[QUERY] {query}")

    try:
        cursor.execute(query)
        result = cursor.fetchone()
    except sqlite3.OperationalError as e:
        conn.close()
        return None, str(e), query

    conn.close()
    return result, None, query


def get_user_messages(username):
    """Fetch inbox for a given username."""
    conn = sqlite3.connect(DB_PATH)
    cursor = conn.cursor()
    cursor.execute(
        "SELECT sender, subject, body, sent_at FROM messages WHERE recipient=? ORDER BY sent_at DESC",
        (username,)
    )
    rows = cursor.fetchall()
    conn.close()
    return rows


def get_all_users():
    """Fetch employee records for the administrative dashboard."""
    conn = sqlite3.connect(DB_PATH)
    cursor = conn.cursor()
    # columns: id, username, email, full_name, department, role, salary, last_login, password
    # password is u[8] in templates
    cursor.execute("SELECT id, username, email, full_name, department, role, salary, last_login, password FROM users")
    rows = cursor.fetchall()
    conn.close()
    return rows


def get_user_by_id(user_id):
    """Fetch an employee record by ID."""
    conn = sqlite3.connect(DB_PATH)
    cursor = conn.cursor()
    cursor.execute(
        "SELECT id, username, email, full_name, department, role, salary, phone, hire_date FROM users WHERE id=?",
        (user_id,)
    )
    row = cursor.fetchone()
    conn.close()
    return row


def get_session_user_id(username):
    """Look up the DB id for the currently logged-in user."""
    conn = sqlite3.connect(DB_PATH)
    cursor = conn.cursor()
    cursor.execute("SELECT id FROM users WHERE username=?", (username,))
    row = cursor.fetchone()
    conn.close()
    return row[0] if row else "?"


def search_users(q):
    """Find employee records matching a username search."""
    conn = sqlite3.connect(DB_PATH)
    cursor = conn.cursor()
    query = f"SELECT username, email, department, role FROM users WHERE username LIKE '%{q}%'"
    print(f"[SEARCH QUERY] {query}")
    try:
        cursor.execute(query)
        results = cursor.fetchall()
        error = None
    except sqlite3.OperationalError as e:
        results = []
        error = str(e)
    conn.close()
    return results, error, query


# ============================================================
# NAVBAR HELPER
# ============================================================

def navbar(role="user"):
    """Returns rendered navbar HTML - use with | safe in templates."""
    admin_link = '<a href="/admin">Admin Panel</a>' if role == "admin" else ""
    label = "🏢 Corp Portal"
    if role == "admin":
        label += ' <span style="color:#e74c3c; font-size:12px;">ADMIN</span>'
    return f"""
    <div class="navbar">
        <span class="nav-left">{label}</span>
        <div>
            <a href="/dashboard">Dashboard</a>
            <a href="/profile?id=1">Profiles</a>
            <a href="/search">Search</a>
            {admin_link}
            <a href="/logout">Logout</a>
        </div>
    </div>
    """


# ============================================================
# TEMPLATES
# ============================================================

BASE_STYLE = """
<style>
    * { box-sizing: border-box; margin: 0; padding: 0; }
    body { font-family: 'Segoe UI', Arial, sans-serif; background: #f0f2f5; color: #222; }
    .navbar {
        background: #1a1a2e; color: white; padding: 14px 30px;
        display: flex; justify-content: space-between; align-items: center;
    }
    .navbar a { color: #aad4f5; text-decoration: none; margin-left: 20px; font-size: 14px; }
    .navbar a:hover { color: white; }
    .container { max-width: 960px; margin: 40px auto; padding: 0 20px; }
    .card {
        background: white; border-radius: 8px; padding: 24px;
        margin-bottom: 20px; box-shadow: 0 2px 8px rgba(0,0,0,0.08);
    }
    h2 { margin-bottom: 16px; color: #1a1a2e; }
    h3 { margin-bottom: 12px; color: #333; }
    table { width: 100%; border-collapse: collapse; font-size: 14px; }
    th { background: #1a1a2e; color: white; padding: 10px 12px; text-align: left; }
    td { padding: 9px 12px; border-bottom: 1px solid #eee; }
    tr:hover td { background: #f9f9f9; }
    .badge {
        display: inline-block; padding: 3px 10px; border-radius: 12px;
        font-size: 12px; font-weight: bold;
    }
    .badge-admin { background: #fde8e8; color: #c0392b; }
    .badge-user  { background: #e8f4fd; color: #2980b9; }
    .info-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 12px; }
    .info-item label { font-size: 12px; color: #888; display: block; margin-bottom: 2px; }
    .info-item span  { font-size: 15px; font-weight: 500; }
    .msg { background: #f8f8f8; border-left: 3px solid #1a1a2e; padding: 12px; margin-bottom: 12px; border-radius: 4px; }
    .msg .meta { font-size: 12px; color: #888; margin-bottom: 4px; }
    .error { color: #c0392b; background: #fde8e8; padding: 10px; border-radius: 4px; margin-top: 10px; }
    input[type=text], input[type=password] {
        width: 100%; padding: 10px; margin: 6px 0 14px;
        border: 1px solid #ddd; border-radius: 4px; font-size: 14px;
    }
    button, .btn {
        padding: 10px 20px; background: #1a1a2e; color: white;
        border: none; border-radius: 4px; cursor: pointer; font-size: 14px;
        text-decoration: none; display: inline-block;
    }
    button:hover, .btn:hover { background: #16213e; }
    .search-bar { display: flex; gap: 10px; margin-bottom: 20px; }
    .search-bar input { margin: 0; flex: 1; }
    .pw { color: #c0392b; font-family: monospace; }
    .nav-left  { font-weight: bold; font-size: 16px; }
    .pagination { margin-top: 16px; display: flex; gap: 8px; }
</style>
"""

LOGIN_PAGE = BASE_STYLE + '''
<div style="max-width:400px; margin:80px auto; padding: 0 20px;">
    <div class="card">
        <h2>🏢 Corp Portal Login</h2>
        <form method="POST">
            <label style="font-size:13px;color:#666;">Username</label>
            <input type="text" name="username" placeholder="username">
            <label style="font-size:13px;color:#666;">Password</label>
            <input type="password" name="password" placeholder="password">
            <button type="submit" style="width:100%;">Sign In</button>
        </form>
        {% if error %}    <div class="error">{{ error }}</div>    {% endif %}
        {% if db_error %} <div class="error"><strong>DB Error:</strong> {{ db_error }}</div> {% endif %}
    </div>
</div>
'''

USER_DASHBOARD = BASE_STYLE + '''
{{ navbar | safe }}
<div class="container">
    <div class="card">
        <h2>Welcome back, {{ full_name }} 👋</h2>
        <div class="info-grid">
            <div class="info-item"><label>Username</label><span>{{ username }}</span></div>
            <div class="info-item"><label>Email</label><span>{{ email }}</span></div>
            <div class="info-item"><label>Department</label><span>{{ department }}</span></div>
            <div class="info-item"><label>Role</label>
                <span><span class="badge badge-{{ role }}">{{ role }}</span></span>
            </div>
            <div class="info-item"><label>Employee Since</label><span>{{ hire_date }}</span></div>
            <div class="info-item"><label>Phone</label><span>{{ phone }}</span></div>
            <div class="info-item"><label>Last Login</label><span>{{ last_login }}</span></div>
            <div class="info-item"><label>Salary</label><span>${{ "{:,}".format(salary) }}</span></div>
        </div>
    </div>
    <div class="card">
        <h3>📬 Inbox</h3>
        {% if messages %}
            {% for msg in messages %}
            <div class="msg">
                <div class="meta">From: <strong>{{ msg[0] }}</strong> &nbsp;·&nbsp; {{ msg[3] }}</div>
                <div><strong>{{ msg[1] }}</strong></div>
                <div style="margin-top:6px; color:#444; font-size:14px;">{{ msg[2] }}</div>
            </div>
            {% endfor %}
        {% else %}
            <p style="color:#888;">No messages.</p>
        {% endif %}
    </div>
</div>
'''

ADMIN_DASHBOARD = BASE_STYLE + '''
{{ navbar | safe }}
<div class="container">
    <div class="card">
        <h2>🛡️ Admin Dashboard - {{ full_name }}</h2>
        <div class="info-grid">
            <div class="info-item"><label>Username</label><span>{{ username }}</span></div>
            <div class="info-item"><label>Email</label><span>{{ email }}</span></div>
            <div class="info-item"><label>Department</label><span>{{ department }}</span></div>
            <div class="info-item"><label>Last Login</label><span>{{ last_login }}</span></div>
        </div>
    </div>
    <div class="card">
        <h3>📬 Inbox</h3>
        {% if messages %}
            {% for msg in messages %}
            <div class="msg">
                <div class="meta">From: <strong>{{ msg[0] }}</strong> &nbsp;·&nbsp; {{ msg[3] }}</div>
                <div><strong>{{ msg[1] }}</strong></div>
                <div style="margin-top:6px; color:#444; font-size:14px;">{{ msg[2] }}</div>
            </div>
            {% endfor %}
        {% else %}
            <p style="color:#888;">No messages.</p>
        {% endif %}
    </div>
    <div class="card">
        <h3>👥 All Employees</h3>
        <table>
            <tr>
                <th>ID</th><th>Username</th><th>Password</th><th>Email</th>
                <th>Name</th><th>Department</th><th>Role</th>
                <th>Salary</th><th>Last Login</th>
            </tr>
            {% for u in all_users %}
            <tr>
                <td>{{ u[0] }}</td>
                <td><a href="/profile?id={{ u[0] }}" style="color:#2980b9;">{{ u[1] }}</a></td>
                <td class="pw">{{ u[8] }}</td>
                <td>{{ u[2] }}</td>
                <td>{{ u[3] }}</td>
                <td>{{ u[4] }}</td>
                <td><span class="badge badge-{{ u[5] }}">{{ u[5] }}</span></td>
                <td>${{ "{:,}".format(u[6]) }}</td>
                <td>{{ u[7] }}</td>
            </tr>
            {% endfor %}
        </table>
    </div>
</div>
'''

PROFILE_PAGE = BASE_STYLE + '''
{{ navbar | safe }}
<div class="container">
    <div class="card">
        <h2>👤 Employee Profile</h2>
        {% if user %}
        <div class="info-grid">
            <div class="info-item"><label>ID</label><span>{{ user[0] }}</span></div>
            <div class="info-item"><label>Username</label><span>{{ user[1] }}</span></div>
            <div class="info-item"><label>Email</label><span>{{ user[2] }}</span></div>
            <div class="info-item"><label>Full Name</label><span>{{ user[3] }}</span></div>
            <div class="info-item"><label>Department</label><span>{{ user[4] }}</span></div>
            <div class="info-item"><label>Role</label>
                <span><span class="badge badge-{{ user[5] }}">{{ user[5] }}</span></span>
            </div>
            <div class="info-item"><label>Salary</label><span>${{ "{:,}".format(user[6]) }}</span></div>
            <div class="info-item"><label>Phone</label><span>{{ user[7] }}</span></div>
            <div class="info-item"><label>Hire Date</label><span>{{ user[8] }}</span></div>
        </div>
        <div class="pagination">
            {% if user[0] > 1 %}
                <a class="btn" href="/profile?id={{ user[0] - 1 }}">← Previous</a>
            {% endif %}
            <a class="btn" href="/profile?id={{ user[0] + 1 }}">Next →</a>
        </div>
        {% else %}
        <div class="error">No user found with ID {{ user_id }}. Try a different ID.</div>
        <div class="pagination">
            {% if user_id > 1 %}
                <a class="btn" href="/profile?id={{ user_id - 1 }}">← Previous</a>
            {% endif %}
            <a class="btn" href="/profile?id={{ user_id + 1 }}">Next →</a>
        </div>
        {% endif %}
    </div>
    <div class="card">
        <h3>🔗 Quick Jump</h3>
        <form method="GET" action="/profile">
            <div class="search-bar">
                <input type="text" name="id" placeholder="Enter user ID..." style="margin:0;">
                <button type="submit">Go</button>
            </div>
        </form>
    </div>
</div>
'''

SEARCH_PAGE = BASE_STYLE + '''
{{ navbar | safe }}
<div class="container">
    <div class="card">
        <h2>🔍 Employee Search</h2>
        <p style="font-size:13px; color:#888; margin-bottom:16px;">
            Search employees by username.
        </p>
        <form method="GET" action="/search">
            <div class="search-bar">
                <input type="text" name="q" value="{{ query }}" placeholder="Search username...">
                <button type="submit">Search</button>
            </div>
        </form>
        {% if db_error %}
            <div class="error"><strong>DB Error:</strong> {{ db_error }}</div>
        {% endif %}
        {% if results %}
        <table>
            <tr><th>Username</th><th>Email</th><th>Department</th><th>Role</th></tr>
            {% for r in results %}
            <tr>
                <td>{{ r[0] }}</td>
                <td>{{ r[1] }}</td>
                <td>{{ r[2] }}</td>
                <td>{{ r[3] }}</td>
            </tr>
            {% endfor %}
        </table>
        {% elif query %}
            <p style="color:#888; margin-top:12px;">No results found.</p>
        {% endif %}
    </div>
</div>
'''

ADMIN_PANEL = BASE_STYLE + '''
{{ navbar | safe }}
<div class="container">
    <div class="card">
        <h2>⚙️ Admin Panel</h2>
    </div>
    <div class="card">
        <h3>👥 All Users</h3>
        <table>
            <tr><th>ID</th><th>Username</th><th>Password</th><th>Email</th><th>Role</th><th>Salary</th></tr>
            {% for u in all_users %}
            <tr>
                <td>{{ u[0] }}</td>
                <td><a href="/profile?id={{ u[0] }}" style="color:#2980b9;">{{ u[1] }}</a></td>
                <td class="pw">{{ u[2] }}</td>
                <td>{{ u[3] }}</td>
                <td><span class="badge badge-{{ u[5] }}">{{ u[5] }}</span></td>
                <td>${{ "{:,}".format(u[6]) }}</td>
            </tr>
            {% endfor %}
        </table>
    </div>
</div>
'''


# ============================================================
# ROUTES
# ============================================================

@app.route("/")
def index():
    return redirect(url_for("login"))


@app.route("/login", methods=["GET", "POST"])
def login():
    error    = None
    db_error = None

    if request.method == "POST":
        username   = request.form.get("username", "")
        password   = request.form.get("password", "")
        ip         = request.remote_addr
        user_agent = request.headers.get("User-Agent", "")
        injection  = detect_injection(username, password)

        row, err, query = authenticate_user(username, password)
        success = row is not None

        log_attempt(ip, username, password, user_agent, success, query, injection)

        if err:
            db_error = err
        elif row:
            # row columns: id, username, password, role, email, full_name, department, hire_date, salary, phone, last_login
            session["username"]   = row[1]
            session["role"]       = row[3]
            session["email"]      = row[4]
            session["full_name"]  = row[5]
            session["department"] = row[6]
            session["hire_date"]  = row[7]
            session["salary"]     = row[8]
            session["phone"]      = row[9]
            session["last_login"] = row[10]
            return redirect(url_for("dashboard"))
        elif not row:
            # Determine the account lookup result.
            conn = sqlite3.connect(DB_PATH)
            cursor = conn.cursor()
            cursor.execute("SELECT id FROM users WHERE username=?", (username,))
            exists = cursor.fetchone()
            conn.close()
            if not exists:
                error = "User not found."
            else:
                error = "Wrong password."

    return render_template_string(LOGIN_PAGE, error=error, db_error=db_error)


@app.route("/dashboard")
def dashboard():
    username = session.get("username", "guest")
    role     = session.get("role", "user")
    messages = get_user_messages(username)

    ctx = {
        "navbar":     navbar(role),
        "username":   username,
        "role":       role,
        "full_name":  session.get("full_name",  "Guest User"),
        "email":      session.get("email",       "guest@corp-internal.com"),
        "department": session.get("department",  "Unknown"),
        "hire_date":  session.get("hire_date",   "N/A"),
        "salary":     session.get("salary",       0),
        "phone":      session.get("phone",        "N/A"),
        "last_login": session.get("last_login",   "N/A"),
        "messages":   messages,
    }

    if role == "admin":
        ctx["all_users"] = get_all_users()
        return render_template_string(ADMIN_DASHBOARD, **ctx)
    else:
        return render_template_string(USER_DASHBOARD, **ctx)


@app.route("/profile")
def profile():
    user_id          = request.args.get("id", 1, type=int)
    user             = get_user_by_id(user_id)
    session_username = session.get("username", "guest")
    session_id       = get_session_user_id(session_username)
    role             = session.get("role", "user")

    return render_template_string(
        PROFILE_PAGE,
        navbar=navbar(role),
        user=user,
        user_id=user_id,
        session_username=session_username,
        session_id=session_id,
    )


@app.route("/admin")
def admin_panel():
    conn = sqlite3.connect(DB_PATH)
    cursor = conn.cursor()
    cursor.execute("SELECT id, username, password, email, full_name, role, salary FROM users")
    all_users = cursor.fetchall()
    conn.close()
    return render_template_string(ADMIN_PANEL, navbar=navbar("admin"), all_users=all_users)


@app.route("/search")
def search():
    q        = request.args.get("q", "")
    results  = []
    db_error = None
    query    = ""
    role     = session.get("role", "user")

    if q:
        results, db_error, query = search_users(q)

    return render_template_string(
        SEARCH_PAGE,
        navbar=navbar(role),
        results=results,
        db_error=db_error,
        query=q,
    )


@app.route("/logout")
def logout():
    session.clear()
    return redirect(url_for("login"))


# ============================================================
# MAIN
# ============================================================

if __name__ == "__main__":
    init_db()
    app.run(host="0.0.0.0", port=5000, debug=True)
