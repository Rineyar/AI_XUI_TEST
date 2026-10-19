# DNS Resolver Vulnerabilities

Vulnerable implementation: `dns/server.py`.

| Vulnerability | Code location | Impact |
|---|---|---|
| Insufficient DNS response validation | Response handling after `sock.recvfrom()` | Only the transaction ID is checked. The response question, record type, and answer relevance are not verified, and DNSSEC validation is absent. An accepted forged upstream response can poison the cache. |
| Small transaction ID range | `randint(10000, 10050)` | Only 51 possible IDs make response guessing easier. |
| Fixed upstream source port | `sock.bind(('10.0.0.2', 22222))` | A predictable port reduces uncertainty for forged responses and creates conflicts between concurrent requests. |
| Missing cache expiration | `Cache.get()` and `Cache.set()` | Cached records never expire, and TTLs are not reduced. Incorrect or poisoned records persist until restart. |
| Incomplete cache keys | Cache indexed only by `qname` | Record type and class are ignored, so an A response can be returned for an AAAA question for the same name. |
| Missing resource limits and cleanup | Upstream socket and cache handling | No upstream timeout, explicit socket closure, or cache size bound is implemented. Requests can wait indefinitely and consume resources. |

The connected UDP socket filters the expected upstream peer, but this does not authenticate packets with a forged source address.

An additional correctness bug overwrites the client's `address` with `(ns, 53)`. On a cache miss, the answer is sent to the upstream server instead of the requesting client.

The source retains hardcoded network addresses and depends on `dnslib` and `gevent`. Deployment files and exploitation tooling are not included.
