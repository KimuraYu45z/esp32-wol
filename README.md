# esp32-wol

Generated with <https://github.com/esp-rs/esp-idf-template>.

## Init

```
cp ./src/config.rs.template ./src/config.rs
```

## Google Apps Script

```javascript
function getMainA1() {
  let ss = SpreadsheetApp.getActiveSpreadsheet();
  let sheet = ss.getSheetByName("main");
  let value = sheet.getRange("A1").getValue();

  return !!value;
}

function setMainA1(value) {
  let ss = SpreadsheetApp.getActiveSpreadsheet();
  let sheet = ss.getSheetByName("main");
  sheet.getRange("A1").setValue(value);
}

function get() {
  return getMainA1();
}

function post() {
  let mainA1 = getMainA1();

  let ss = SpreadsheetApp.getActiveSpreadsheet();
  let sheet = ss.getSheetByName("logs");
  sheet.insertRowAfter(1);
  let formattedDate = Utilities.formatDate(new Date(), "GMT", "yyyy-MM-dd'T'HH:mm:ss'Z'");
  sheet.getRange("A2").setValue([formattedDate, mainA1]);

  setMainA1("");

  return true;
}
```
