# Setup Flow

## User completes first-time setup successfully

* Open the WoW on Silicon app
* Wait for app to be ready
* Select runner "CrossOver" from the runner dropdown
* Browse and select WoW folder "/tmp/test-wow"
* Verify validation shows "WoW installation verified"
* Click the "Setup" button
* Wait for setup to complete
* Verify alert "Setup complete" is displayed
* Verify "Run" button is visible

## User sees error for invalid game folder

* Open the WoW on Silicon app
* Wait for app to be ready
* Select runner "Whisky" from the runner dropdown
* Browse and select folder "/tmp/empty-dir"
* Verify validation shows error "WoW.exe not found"
* Verify "Setup" button is disabled
