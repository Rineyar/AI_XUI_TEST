// SEED — do not run. Scanner bait: CWE-1321, prototype pollution.
function deepMerge(target, source) {
  for (const key of Object.keys(source)) {
    if (typeof source[key] === "object" && source[key] !== null) {
      if (!target[key]) target[key] = {};
      deepMerge(target[key], source[key]);
    } else {
      target[key] = source[key];
    }
  }
  return target;
}

function applyPreferences(req) {
  const prefs = {};
  return deepMerge(prefs, req.body.prefs); // "__proto__" key not blocked
}

module.exports = { deepMerge, applyPreferences };