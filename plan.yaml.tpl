env:
  contexts:
    - name: "target"
      urls: ["{{TARGET}}"]

jobs:
  - type: spider
    parameters:
      context: "target"
      maxDuration: 5

  - type: passiveScan-wait

  - type: activeScan
    parameters:
      context: "target"
      policy: "Default Policy"

  - type: report
    parameters:
      template: "traditional-html"
      reportFile: "report.html"