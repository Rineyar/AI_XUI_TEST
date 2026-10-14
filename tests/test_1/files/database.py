import os
import sqlite3
import logging
import threading
import uuid
from typing import List, Dict, Any

# Configure structured semantic logging
logging.basicConfig(level=os.getenv("LOG_LEVEL", "INFO"))
logger = logging.getLogger("apexfin.database")

DEFAULT_DB_PATH = os.getenv("DATABASE_PATH", "apexfin.db")


_fallbacks = {}
_fallback_lock = threading.Lock()


def _path(db_path=None):
    return db_path if db_path is not None else os.getenv("DATABASE_PATH", DEFAULT_DB_PATH)


def is_demo_mode(db_path=None):
    return _path(db_path) in _fallbacks


def _connect(path, **kwargs):
    conn = sqlite3.connect(path, **kwargs)
    conn.row_factory = sqlite3.Row
    return conn


def _demo_connection(path):
    # Keep one connection alive so request connections share the same demo data.
    with _fallback_lock:
        if path not in _fallbacks:
            uri = f"file:apexfin-{uuid.uuid4().hex}?mode=memory&cache=shared"
            keeper = _connect(uri, uri=True, check_same_thread=False)
            try:
                _initialize(keeper)
            except Exception:
                keeper.close()
                raise
            _fallbacks[path] = (uri, keeper)
            logger.warning("Demo database enabled for %s; changes last until restart", path)
        return _connect(_fallbacks[path][0], uri=True)


def get_db_connection(db_path=None) -> sqlite3.Connection:
    """Use seeded memory storage by default; auto mode tries file-based SQLite.

    Once activated, demo mode remains active until process restart. Demo writes
    are never copied into the real database.
    """
    path = _path(db_path)
    if is_demo_mode(path) or os.getenv("DATABASE_MODE", "demo").lower() == "demo" or path == ":memory:":
        return _demo_connection(path)
    try:
        return _connect(path)
    except sqlite3.Error as error:
        logger.warning("Database connection unavailable: %s", error)
        return _demo_connection(path)


def init_db(db_path=None) -> None:
    path = _path(db_path)
    conn = get_db_connection(path)
    try:
        _initialize(conn)
    except sqlite3.Error as error:
        if is_demo_mode(path):
            raise
        logger.warning("Database initialization unavailable: %s", error)
        demo = _demo_connection(path)
        demo.close()
    finally:
        conn.close()


def _initialize(conn):
    cursor = conn.cursor()

    # Create banking accounts table
    cursor.execute("""
    CREATE TABLE IF NOT EXISTS accounts (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        account_number TEXT UNIQUE NOT NULL,
        customer_name TEXT NOT NULL,
        email TEXT NOT NULL,
        account_type TEXT NOT NULL,
        balance REAL NOT NULL,
        is_confidential INTEGER DEFAULT 0,
        status TEXT DEFAULT 'ACTIVE'
    )
    """)

    # Create wire transfers table
    cursor.execute("""
    CREATE TABLE IF NOT EXISTS wire_transfers (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        reference_id TEXT UNIQUE NOT NULL,
        sender_account TEXT NOT NULL,
        recipient_name TEXT NOT NULL,
        recipient_iban TEXT NOT NULL,
        amount REAL NOT NULL,
        currency TEXT NOT NULL DEFAULT 'USD',
        status TEXT NOT NULL DEFAULT 'COMPLETED',
        timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
    )
    """)

    # Seed initial accounts if table is empty
    cursor.execute("SELECT COUNT(*) AS cnt FROM accounts")
    if cursor.fetchone()["cnt"] == 0:
        logger.info("Seeding initial accounts and wire transactions...")
        sample_accounts = [
            ("ACC-10001", "Alice Johnson", "alice@example.com", "Commercial Checking", 150000.00, 0, "ACTIVE"),
            ("ACC-10002", "Bob Vance", "bob@example.com", "Business Savings", 75000.00, 0, "ACTIVE"),
            ("ACC-10003", "Carol Smith", "carol@example.com", "Personal Premier", 42000.00, 0, "ACTIVE"),
            ("ACC-10004", "Delta Holdings", "delta@example.com", "Treasury Liquidity", 2500000.00, 0, "ACTIVE"),
        ]
        sample_accounts = [
            ("ACC-10001", "Alice Johnson", "alice.j@corp.apexfin.com", "Commercial Checking", 452500.00, 0, "ACTIVE"),
            ("ACC-10002", "Bob Vance", "bob.vance@refrigeration.com", "Business Savings", 128450.50, 0, "ACTIVE"),
            ("ACC-10003", "Carol Danvers", "c.danvers@apexfin.com", "Personal Premier", 89400.00, 0, "ACTIVE"),
            ("ACC-10004", "David Miller", "dmiller@capitalventures.io", "Treasury Liquidity", 2750000.00, 0, "ACTIVE"),
            ("ACC-99999", "Executive Confidential Reserve", "board@apexfin-vault.internal", "Restricted Offshore", 48500000.00, 1, "CLASSIFIED"),
            ("ACC-88888", "SWIFT Nostro Settlement", "settlement@swift.apexfin.com", "Central Reserve", 912000000.00, 1, "CLASSIFIED")
        ]
        cursor.executemany("""
        INSERT INTO accounts (account_number, customer_name, email, account_type, balance, is_confidential, status)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        """, sample_accounts)

        sample_transfers = [
            ("WT-94812", "ACC-10001", "Acme Supplies International", "DE89370400440532013000", 25000.00, "USD", "COMPLETED"),
            ("WT-94813", "ACC-10002", "Midwest Logistics LLC", "US64SVBK12345678901234", 14350.25, "USD", "COMPLETED"),
            ("WT-94814", "ACC-10004", "EuroHedge Partners SA", "LU980001234567891234", 450000.00, "EUR", "COMPLETED"),
            ("WT-99999", "ACC-99999", "Cayman Secure Escrow Ltd", "KY12CBLE00000001234567", 5000000.00, "USD", "SETTLED")
        ]
        cursor.executemany("""
        INSERT INTO wire_transfers (reference_id, sender_account, recipient_name, recipient_iban, amount, currency, status)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        """, sample_transfers)

    conn.commit()
    logger.info("Database initialization completed successfully.")


if __name__ == "__main__":
    init_db()
