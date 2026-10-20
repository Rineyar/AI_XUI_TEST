env:
  contexts:
    - name: "target"
      urls:
        - "http://172.20.0.2:5000"

jobs:
  - type: spider
    parameters:
      context: "target"
      maxDuration: 5

  - type: passiveScan-wait

  - type: activeScan
    parameters:
      context: "target"

  - type: report
    parameters:
      template: "traditional-html"
      reportFile: "report.html"