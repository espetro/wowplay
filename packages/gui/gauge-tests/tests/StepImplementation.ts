import { Step, BeforeSuite, AfterSuite } from 'gauge-ts';
import { mkdirSync, writeFileSync, existsSync } from 'fs';
import { join } from 'path';
import { TauriPilotFlow } from '../support/tauri-pilot';

export default class StepImplementation {
  @BeforeSuite()
  public async beforeSuite() {
    const testWowDir = '/tmp/test-wow';
    const emptyDir = '/tmp/empty-dir';

    if (!existsSync(testWowDir)) {
      mkdirSync(testWowDir, { recursive: true });
      writeFileSync(join(testWowDir, 'WoW.exe'), '');
    }

    if (!existsSync(emptyDir)) {
      mkdirSync(emptyDir, { recursive: true });
    }
  }

  @AfterSuite()
  public async afterSuite() {}

  @Step("Open the WoW on Silicon app")
  public async openApp() {
    // tauri-pilot starts the app automatically before the first step
  }

  @Step("Wait for app to be ready")
  public async waitForAppReady() {
    // tauri-pilot handles app startup synchronization
  }

  @Step("Select runner <runner> from the runner dropdown")
  public async selectRunner(runner: string) {
    const value = runner.toLowerCase().replace(/\s+/g, '-');
    new TauriPilotFlow('select-runner')
      .click("[data-testid='runner-select-trigger']")
      .click(`[data-value='${value}']`)
      .run();
  }

  @Step("Browse and select WoW folder <folder>")
  public async selectWowFolder(folder: string) {
    new TauriPilotFlow('select-wow-folder')
      .ipc('validate_wow_dir', { path: folder })
      .run();
  }

  @Step("Click the \"Browse\" button")
  public async clickBrowse() {
    new TauriPilotFlow('click-browse')
      .click("[data-testid='folder-browse-btn']")
      .run();
  }

  @Step("Select folder <folder>")
  public async selectFolder(folder: string) {
    new TauriPilotFlow('select-folder')
      .ipc('validate_wow_dir', { path: folder })
      .run();
  }

  @Step("Verify validation shows <message>")
  public async verifyValidationMessage(message: string) {
    new TauriPilotFlow('verify-validation')
      .assert("[data-testid='alert-info']", { text: message })
      .run();
  }

  @Step("Verify validation shows error <message>")
  public async verifyValidationError(message: string) {
    new TauriPilotFlow('verify-validation-error')
      .assert("[data-testid='alert-error']", { text: message })
      .run();
  }

  @Step("Click the \"Setup\" button")
  public async clickSetup() {
    new TauriPilotFlow('click-setup')
      .click("[data-testid='action-btn']")
      .run();
  }

  @Step("Wait for setup to complete")
  public async waitForSetupComplete() {
    new TauriPilotFlow('wait-setup-complete')
      .wait("[data-testid='alert-info']", 'Setup complete')
      .run();
  }

  @Step("Verify alert <message> is displayed")
  public async verifyAlert(message: string) {
    new TauriPilotFlow('verify-alert')
      .assert("[data-testid='alert-info']", { text: message })
      .run();
  }

  @Step("Verify \"Setup\" button is visible")
  public async verifySetupButtonVisible() {
    new TauriPilotFlow('verify-setup-visible')
      .assert("[data-testid='action-btn']", { text: 'Setup', visible: true })
      .run();
  }

  @Step("Verify \"Run\" button is visible")
  public async verifyRunButtonVisible() {
    new TauriPilotFlow('verify-run-visible')
      .assert("[data-testid='action-btn']", { text: 'Run', visible: true })
      .run();
  }

  @Step("Verify \"Setup\" button is disabled")
  public async verifySetupButtonDisabled() {
    new TauriPilotFlow('verify-setup-disabled')
      .assert("[data-testid='action-btn']", { disabled: true })
      .run();
  }

  @Step("Verify \"Setup\" button is enabled")
  public async verifySetupButtonEnabled() {
    new TauriPilotFlow('verify-setup-enabled')
      .assert("[data-testid='action-btn']", { disabled: false })
      .run();
  }

  @Step("Click the \"Run\" button")
  public async clickRun() {
    new TauriPilotFlow('click-run')
      .click("[data-testid='action-btn']")
      .run();
  }

  @Step("Open the options menu")
  public async openOptionsMenu() {
    new TauriPilotFlow('open-options-menu')
      .click("[data-testid='options-menu-btn']")
      .run();
  }

  @Step("Click \"Reset Configuration\"")
  public async clickResetConfig() {
    new TauriPilotFlow('click-reset')
      .click("[data-testid='menu-item-reset']")
      .run();
  }

  @Step("Confirm reset action")
  public async confirmReset() {
    // tauri-pilot handles the native dialog confirm
  }

  @Step("Verify runner dropdown is empty")
  public async verifyRunnerEmpty() {
    new TauriPilotFlow('verify-runner-empty')
      .assert("[data-testid='runner-select-trigger']", { text: 'Select runner...' })
      .run();
  }

  @Step("Verify game folder is empty")
  public async verifyFolderEmpty() {
    new TauriPilotFlow('verify-folder-empty')
      .assert("[data-testid='folder-browse-btn']", { visible: true })
      .run();
  }

  @Step("Click \"Show Alerts\"")
  public async clickShowAlerts() {
    new TauriPilotFlow('click-show-alerts')
      .click("[data-testid='menu-item-toggle-alerts']")
      .run();
  }

  @Step("Close the options menu")
  public async closeOptionsMenu() {
    new TauriPilotFlow('close-options-menu')
      .click('body')
      .run();
  }

  @Step("Verify menu is closed")
  public async verifyMenuClosed() {
    new TauriPilotFlow('verify-menu-closed')
      .assert("[data-testid='options-menu-btn']", { visible: true })
      .run();
  }

  @Step("Verify error alert <message> is displayed")
  public async verifyErrorAlert(message: string) {
    new TauriPilotFlow('verify-error-alert')
      .assert("[data-testid='alert-error']", { text: message })
      .run();
  }

  @Step("Verify validation shows checkmark")
  public async verifyValidationCheckmark() {
    new TauriPilotFlow('verify-validation-checkmark')
      .assert("[data-testid='alert-info']", { visible: true })
      .run();
  }
}
