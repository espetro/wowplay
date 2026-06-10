# Browse and Validate Flow

## User browses and selects valid WoW folder

Tags: browse-flow

* Open the WoW on Silicon app
* Wait for app to be ready
* Click the "Browse" button
* Select folder "/tmp/test-wow"
* Verify validation shows checkmark
* Verify "Setup" button is enabled

## User selects invalid folder

Tags: browse-flow, validation

* Open the WoW on Silicon app
* Wait for app to be ready
* Click the "Browse" button
* Select folder "/tmp/empty-dir"
* Verify error alert "WoW.exe not found" is displayed
* Verify "Setup" button is disabled
