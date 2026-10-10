1) Perform reconnaissance of the target web application:
   - Confirm the target is reachable and responds over HTTP/HTTPS.
   - Identify the base URL, tech stack hints (headers, cookies, error pages).
   - Record any authentication requirements or session handling observed.

2) Run Nuclei against the target:
   1. Use the `run_nuclei` tool with the target URL and default severity range
      (`info,low,medium,high,critical`).
   2. If a template or a narrower severity is required, pass it explicitly.
   3. Collect and store the raw JSON findings.

3) Run OWASP ZAP against the target:
   1. Use the `run_zap` tool with the target URL.
   2. Start with passive mode (`do_ascan=False`) to enumerate the application
      via the spider and collect passive alerts.
   3. If the spider discovered enough endpoints and the scope allows active
      testing, re-run with `do_ascan=True` to enable active scanning.
   4. Collect and store the raw alerts JSON.

4) Analyze the collected results:
   1. Parse the Nuclei output and the ZAP alerts.
   2. Merge duplicates found by both tools.
   3. Filter out obvious false positives (e.g. informational headers that are
      expected for the stack, template mismatches).
   4. Map each finding to a category from OWASP Top 10 where possible.

5) Perform an additional manual review:
   1. Check the discovered endpoints for common issues the tools may have
      missed (auth bypass, IDOR, business logic flaws).
   2. Verify the most critical findings by reproducing them if the scope
      permits.

6) Produce a report on the security state of the target:
   - For each finding include: name, severity, affected URL/parameter,
     evidence, short description, and remediation hint.
   - Rank findings by severity (critical > high > medium > low > info).
   - Summarize the overall posture and list the top priorities.

