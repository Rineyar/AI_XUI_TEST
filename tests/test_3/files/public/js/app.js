// Front-end glue. VULN (A05/A07): the API token is kept in localStorage, so
// any stored XSS on the shop pages can exfiltrate it.
// SAFE: do not persist bearer tokens where scripts can read them; keep the
// session cookie HttpOnly and let the server render what the page needs.

(function () {
  "use strict";

  /* ---------- API token (unchanged — the XSS lab depends on this) ---------- */

  var token = localStorage.getItem("nodebazaar_token");
  if (!token) {
    var el = document.getElementById("api-token");
    if (el) {
      token = el.textContent.trim();
      try { localStorage.setItem("nodebazaar_token", token); } catch (e) {}
    }
  }
  var preview = document.getElementById("api-me");
  if (preview && token) {
    fetch("/api/v1/me", { headers: { Authorization: "Bearer " + token } })
      .then(function (r) { return r.json(); })
      .then(function (u) {
        preview.textContent = u.username ? (u.username + " (" + u.role + ")") : "token rejected";
      })
      .catch(function () { preview.textContent = "token rejected"; });
  }

  /* ---------- Live search dropdown ---------- */

  var box = document.querySelector(".searchbox");
  if (box) {
    var input = box.querySelector("input[name=q]");
    var wrap = box.querySelector(".searchbox-wrap");
    var panel = null;
    var timer = null;
    var seq = 0;

    function close() {
      if (panel) { panel.remove(); panel = null; }
    }

    function render(items, n) {
      if (n !== seq) return; // stale response
      close();
      if (!items.length) return;
      panel = document.createElement("div");
      panel.className = "search-suggest";
      items.forEach(function (p) {
        var a = document.createElement("a");
        a.href = "/products/" + p.id;
        var name = document.createElement("span");
        name.className = "ss-name";
        name.textContent = p.name;
        var price = document.createElement("span");
        price.className = "ss-price";
        price.textContent = (p.price_cents / 100).toFixed(2) + " €";
        a.appendChild(name);
        a.appendChild(price);
        panel.appendChild(a);
      });
      wrap.appendChild(panel);
    }

    input.addEventListener("input", function () {
      var q = input.value.trim();
      clearTimeout(timer);
      if (q.length < 2) { close(); return; }
      timer = setTimeout(function () {
        var n = ++seq;
        fetch("/search?q=" + encodeURIComponent(q), { headers: { Accept: "text/html" } })
          .then(function (r) { return r.text(); })
          .then(function (html) {
            var doc = new DOMParser().parseFromString(html, "text/html");
            var items = Array.prototype.map.call(doc.querySelectorAll(".grid .card"), function (card) {
              var img = card.querySelector("img");
              return {
                id: (card.getAttribute("href") || "").split("/").pop(),
                name: img ? img.getAttribute("alt") : card.querySelector("h3").textContent,
                price_cents: Math.round(parseFloat(
                  (card.querySelector(".price").textContent || "0").replace(/[^\d.]/g, "")
                ) * 100)
              };
            });
            render(items, n);
          })
          .catch(function () {});
      }, 250);
    });

    input.addEventListener("keydown", function (e) {
      if (e.key === "Escape") close();
    });
    document.addEventListener("click", function (e) {
      if (!box.contains(e.target)) close();
    });
  }

  /* ---------- Cart badge ---------- */

  var CART_KEY = "nodebazaar_cart";

  function cartCount() {
    var n = parseInt(localStorage.getItem(CART_KEY), 10);
    return isNaN(n) || n < 0 ? 0 : n;
  }

  var badge = document.querySelector(".cart-badge");
  if (badge) {
    function paint() {
      var n = cartCount();
      badge.textContent = n > 99 ? "99+" : String(n);
      badge.hidden = n === 0;
    }
    paint();
    document.addEventListener("submit", function (e) {
      var f = e.target;
      var inBuybox = f.closest && f.closest(".buybox, .checkout-form");
      if (!inBuybox) return;
      try { localStorage.setItem(CART_KEY, String(cartCount() + 1)); } catch (err) {}
      paint();
      // No preventDefault: the POST proceeds normally.
    });
  }

  /* ---------- Scroll-to-top ---------- */

  var toTop = document.querySelector(".to-top");
  if (toTop) {
    var toggle = function () {
      toTop.classList.toggle("show", window.scrollY > 600);
    };
    window.addEventListener("scroll", toggle, { passive: true });
    toggle();
    toTop.addEventListener("click", function () {
      window.scrollTo({ top: 0, behavior: "smooth" });
    });
  }

  /* ---------- Product image fade-in ---------- */

  if ("IntersectionObserver" in window) {
    var io = new IntersectionObserver(function (entries) {
      entries.forEach(function (entry) {
        if (!entry.isIntersecting) return;
        var img = entry.target;
        io.unobserve(img);
        if (img.complete) { img.classList.add("loaded"); return; }
        img.addEventListener("load", function () {
          img.classList.add("loaded");
        }, { once: true });
      });
    });
    document.querySelectorAll(".card img").forEach(function (img) {
      img.classList.add("fade");
      io.observe(img);
    });
  }
})();