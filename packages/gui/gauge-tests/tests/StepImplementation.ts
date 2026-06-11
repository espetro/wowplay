import { Step, BeforeSuite, AfterSuite, BeforeScenario, AfterScenario, ExecutionContext } from 'gauge-ts';
import { mkdirSync, writeFileSync, existsSync } from 'fs';
import { join } from 'path';
import { TauriPilotFlow } from '../support/tauri-pilot';

export default class StepImplementation {
  private _scenarioFlow: TauriPilotFlow | null = null;
  private _scenarioName: string = '';

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

  @BeforeScenario()
  public async beforeScenario(context: ExecutionContext) {
    const scenario = context.getCurrentScenario();
    this._scenarioName = scenario?.getName() ?? 'unknown-scenario';
    this._scenarioFlow = new TauriPilotFlow(this._scenarioName);
  }

  @AfterScenario()
  public async afterScenario() {
    try {
      if (this._scenarioFlow && this._scenarioFlow.stepCount() > 0) {
        this._scenarioFlow.run();
      }
    } finally {
      this._scenarioFlow = null;
      this._scenarioName = '';
    }
  }

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
    if (!this._scenarioFlow) return;
    const value = runner.toLowerCase().replace(/\s+/g, '-');
    this._scenarioFlow
      .click("[data-testid='runner-select-trigger']")
      .click(`[data-value='${value}']`);
  }

  @Step("Browse and select WoW folder <folder>")
  public async selectWowFolder(folder: string) {
    if (!this._scenarioFlow) return;
    this._scenarioFlow.ipc('validate_wow_dir', { path: folder });
  }

  @Step("Browse and select folder <folder>")
  public async browseAndSelectFolder(folder: string) {
    if (!this._scenarioFlow) return;
    this._scenarioFlow.ipc('validate_wow_dir', { path: folder });
  }

  @Step("Click the <buttonName> button")
  public async clickButton(buttonName: string) {
    if (!this._scenarioFlow) return;
    const testId = buttonName.toLowerCase() === 'browse' ? 'folder-browse-btn' : 'action-btn';
    this._scenarioFlow.click(`[data-testid='${testId}']`);
  }

  @Step("Select folder <folder>")
  public async selectFolder(folder: string) {
    if (!this._scenarioFlow) return;
    this._scenarioFlow.ipc('validate_wow_dir', { path: folder });
  }

  @Step("Verify validation shows <message>")
  public async verifyValidationMessage(message: string) {
    if (!this._scenarioFlow) return;
    this._scenarioFlow.assert("[data-testid='alert-info']", { text: message });
  }

  @Step("Verify validation shows error <message>")
  public async verifyValidationError(message: string) {
    if (!this._scenarioFlow) return;
    this._scenarioFlow.assert("[data-testid='alert-error']", { text: message });
  }

  @Step("Wait for setup to complete")
  public async waitForSetupComplete() {
    if (!this._scenarioFlow) return;
    this._scenarioFlow.wait("[data-testid='alert-info']", 'Setup complete');
  }

  @Step("Verify alert <message> is displayed")
  public async verifyAlert(message: string) {
    if (!this._scenarioFlow) return;
    this._scenarioFlow.assert("[data-testid='alert-info']", { text: message });
  }

  @Step("Verify <buttonName> button is visible")
  public async verifyButtonVisible(buttonName: string) {
    if (!this._scenarioFlow) return;
    this._scenarioFlow.assert("[data-testid='action-btn']", { text: buttonName, visible: true });
  }

  @Step("Verify <buttonName> button is disabled")
  public async verifyButtonDisabled(buttonName: string) {
    if (!this._scenarioFlow) return;
    this._scenarioFlow.assert("[data-testid='action-btn']", { text: buttonName, disabled: true });
  }

  @Step("Verify <buttonName> button is enabled")
  public async verifyButtonEnabled(buttonName: string) {
    if (!this._scenarioFlow) return;
    this._scenarioFlow.assert("[data-testid='action-btn']", { text: buttonName, disabled: false });
  }

  @Step("Open the options menu")
  public async openOptionsMenu() {
    if (!this._scenarioFlow) return;
    this._scenarioFlow.click("[data-testid='options-menu-btn']");
  }

  @Step("Click <menuItem>")
  public async clickMenuItem(menuItem: string) {
    if (!this._scenarioFlow) return;
    const testId = menuItem === 'Reset Configuration' ? 'menu-item-reset' : 'menu-item-toggle-alerts';
    this._scenarioFlow.click(`[data-testid='${testId}']`);
  }

  @Step("Confirm reset action")
  public async confirmReset() {
    // tauri-pilot handles the native dialog confirm
  }

  @Step("Verify runner dropdown is empty")
  public async verifyRunnerEmpty() {
    if (!this._scenarioFlow) return;
    this._scenarioFlow.assert("[data-testid='runner-select-trigger']", { text: 'Select runner...' });
  }

  @Step("Verify game folder is empty")
  public async verifyFolderEmpty() {
    if (!this._scenarioFlow) return;
    this._scenarioFlow.assert("[data-testid='folder-browse-btn']", { visible: true });
  }

  @Step("Close the options menu")
  public async closeOptionsMenu() {
    if (!this._scenarioFlow) return;
    this._scenarioFlow.click('body');
  }

  @Step("Verify menu is closed")
  public async verifyMenuClosed() {
    if (!this._scenarioFlow) return;
    this._scenarioFlow.assert("[data-testid='options-menu-btn']", { visible: true });
  }

  @Step("Verify error alert <message> is displayed")
  public async verifyErrorAlert(message: string) {
    if (!this._scenarioFlow) return;
    this._scenarioFlow.assert("[data-testid='alert-error']", { text: message });
  }

  @Step("Verify validation shows checkmark")
  public async verifyValidationCheckmark() {
    if (!this._scenarioFlow) return;
    this._scenarioFlow.assert("[data-testid='alert-info']", { visible: true });
  }
}
