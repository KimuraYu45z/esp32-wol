# esp32-wol

Generated with <https://github.com/esp-rs/esp-idf-template>.

## Google Apps Script

```javascript
function init() {
  return true;
}

function get() {
  let ss = SpreadsheetApp.getActiveSpreadsheet();
  let sheet = ss.getSheetByName("main");
  let value = sheet.getRange("A1").getValue();

  return !!value;
}

function post() {
  let ss = SpreadsheetApp.getActiveSpreadsheet();
  let sheet = ss.getSheetByName("logs");
  sheet.insertRowAfter(1);
  let formattedDate = Utilities.formatDate(new Date(), "GMT", "yyyy-MM-dd'T'HH:mm:ss'Z'");
  sheet.getRange("A2").setValue(formattedDate);

  return true;
}
```
