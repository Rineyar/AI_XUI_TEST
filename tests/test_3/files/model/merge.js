// Naive "deep merge" used by the preferences endpoint. This is the classic
// prototype-pollution gadget: __proto__ in user JSON reaches Object.prototype.
// VULN (A08 Software/Data Integrity Failures): recursive merge without a key
// blocklist lets { "__proto__": { "isAdmin": true } } poison every object.
// SAFE: reject keys named __proto__, constructor, prototype before merging
// (or drop the hand-rolled merge and use a maintained one with that guard).

function merge(target, source) {
  for (const key of Object.keys(source)) {
    if (typeof source[key] === "object" && source[key] !== null && !Array.isArray(source[key])) {
      if (!target[key]) target[key] = {};
      merge(target[key], source[key]);
    } else {
      target[key] = source[key];
    }
  }
  return target;
}

module.exports = { merge };