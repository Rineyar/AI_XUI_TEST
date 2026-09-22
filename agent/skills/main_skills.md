# Main Skills
## Environment Reconnaissance
Before performing analysis, gather only the context necessary for the current task.
- Inspect available files, directories, configuration, or runtime information when relevant.
- Identify potentially relevant targets before performing deeper analysis.
- Avoid unnecessary inspection unrelated to the user's request.

## File and Content Analysis
When analyzing files:
1. Identify files relevant to the task.
2. Inspect their contents using available tools.
3. Look for security-relevant configurations, suspicious patterns, sensitive data exposure, or other issues relevant to the current task.
4. Correlate information from multiple files when necessary.

## Runtime and Environment Analysis
When runtime or environment information is relevant:
- Inspect available runtime information using appropriate tools.
- Look for insecure configuration, unexpected values, exposed information, or other security-relevant conditions.
- Treat unavailable or filtered information as unavailable rather than attempting to infer its contents.

## Network Interaction
When network interaction is required:
1. Determine what information or behavior needs to be verified.
2. Perform only the requests necessary for that verification.
3. Inspect responses, status information, headers, and returned content when relevant.
4. Use observed results as evidence for further analysis.

## Finding Verification
Before reporting a potential security finding:
- Verify it using available evidence when possible.
- Distinguish confirmed behavior from suspicious patterns or hypotheses.
- Correlate multiple sources of information when required.
- Avoid presenting an unverified assumption as a confirmed vulnerability.

## Reporting
When reporting analysis results:
- Describe what was inspected.
- Describe relevant observations and evidence.
- Clearly separate confirmed findings from assumptions or uncertainties.
- Explain the potential security impact when applicable.
- Provide remediation guidance when applicable.
- Mention limitations that prevented complete verification.
