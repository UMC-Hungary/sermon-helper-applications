## ADDED Requirements

### Requirement: Settings overview
The settings screen SHALL open with an at-a-glance summary of the application's operating state, covering at minimum the current mode, how many sign-in accounts are connected against how many are configurable, and how many scheduled jobs are enabled. Each summary value SHALL reflect live state.

#### Scenario: Summary reflects live state
- **WHEN** the settings screen is opened
- **THEN** the summary shows the current mode, account connection counts and enabled job count from live data

#### Scenario: Summary updates
- **WHEN** an account is connected or a job is enabled
- **THEN** the summary updates without a manual refresh

### Requirement: Interface language
Settings SHALL let the user choose the interface language among the supported locales, apply it immediately, and persist it.

#### Scenario: Changing language
- **WHEN** the user selects another language
- **THEN** the interface re-renders in it immediately and the choice persists across restarts

### Requirement: Application mode
Settings SHALL show the current application mode and allow changing it, explaining what changing it entails before the change takes effect.

#### Scenario: Viewing mode
- **WHEN** the settings screen is opened
- **THEN** the current mode is shown with an explanation of its implications

#### Scenario: Changing mode
- **WHEN** the user changes the mode
- **THEN** the consequences are explained and, on confirmation, the change is persisted through the core

### Requirement: Sign-in accounts
Settings SHALL list the publishing platform accounts, showing for each whether it is signed in, whether its credentials are configured, and the signed-in identity where available. It SHALL offer signing in where credentials exist, and account management where already signed in.

#### Scenario: Account states distinguished
- **WHEN** the accounts are listed
- **THEN** each shows whether it is signed in, awaiting sign-in, or missing credentials

#### Scenario: Signing in
- **WHEN** the user signs in to an account with credentials configured
- **THEN** the sign-in flow is presented and, on success, the account shows as connected

#### Scenario: Credentials missing
- **WHEN** an account has no credentials configured
- **THEN** it says so, and sign-in is not offered until they are provided

### Requirement: Scheduled jobs
Settings SHALL list the configured scheduled jobs with their name, schedule expression and purpose, allow each to be enabled or disabled, and allow a new job to be added by name and schedule expression. An invalid schedule expression SHALL be rejected with an explanation.

#### Scenario: Listing and toggling
- **WHEN** jobs are listed
- **THEN** each shows its name, schedule and purpose, and can be enabled or disabled with the change persisted

#### Scenario: Adding a job
- **WHEN** the user supplies a name and a valid schedule expression
- **THEN** the job is created through the core and appears in the list

#### Scenario: Invalid schedule
- **WHEN** the schedule expression is invalid or a field is empty
- **THEN** the job is not created and the problem is explained

### Requirement: Presentation preference
Settings SHALL let the user choose whether the built-in web presenter is used, explaining the effect, and persist the choice through the core.

#### Scenario: Toggling the web presenter
- **WHEN** the user changes the setting
- **THEN** it is sent to the core and the new state is reflected

### Requirement: Appearance
Settings SHALL expose the colour-scheme control, offering light, dark and automatic.

#### Scenario: Changing appearance
- **WHEN** the user changes the colour scheme
- **THEN** the interface adopts it immediately and the choice persists

### Requirement: Version and updates
Settings SHALL show the running application version and let the user check for an update, indicating when one is available and offering to obtain it.

#### Scenario: Checking for updates
- **WHEN** the user checks for updates
- **THEN** the result is reported, and an available update is offered

#### Scenario: Up to date
- **WHEN** no update is available
- **THEN** the current version is shown as current

### Requirement: Connector inventory
Connector management SHALL list the configured connectors grouped by the categories the reference defines, with a summary of how many are live, ready and not yet available. Each connector SHALL show its identity, a status indication, a short description of its current state, and a control to enable or disable it. A connector that is configurable but not yet supported SHALL be shown as such and SHALL NOT be enableable.

#### Scenario: Grouped inventory
- **WHEN** connector management is opened
- **THEN** connectors are listed within their categories, with the live, ready and unavailable summary shown

#### Scenario: Enabling a connector
- **WHEN** the user enables a supported connector
- **THEN** the change is persisted through the core and its status updates

#### Scenario: Unsupported connector
- **WHEN** a connector is configurable but not yet supported
- **THEN** it is marked as such, its configuration is retained, and it cannot be enabled

#### Scenario: Status changes live
- **WHEN** a connector's status changes at the core
- **THEN** its indication updates without a manual refresh

### Requirement: Connector configuration in place
Selecting a connector SHALL reveal its configuration within the list rather than navigating away, showing the fields that connector requires. Secret values SHALL NOT be displayed in readable form. Configuration SHALL be saved through the core.

#### Scenario: Revealing configuration
- **WHEN** the user selects a connector
- **THEN** its configuration fields appear in place, and selecting it again collapses them

#### Scenario: Secrets are not revealed
- **WHEN** a connector has a secret value configured
- **THEN** it is not displayed in readable form

#### Scenario: Saving configuration
- **WHEN** the user saves a connector's configuration
- **THEN** it is persisted through the core and the outcome is reported

### Requirement: Encoder connection and destination routing
The encoder connector SHALL offer connecting and disconnecting, saving configuration and reconnecting, choosing which publishing destination it targets, and viewing that destination's streaming address. Destination selection SHALL be unavailable while the encoder is not connected.

#### Scenario: Connecting
- **WHEN** the user connects the encoder
- **THEN** the connection is attempted through the core and the outcome is reflected in its status

#### Scenario: Choosing a destination
- **WHEN** the encoder is connected and the user chooses a destination
- **THEN** that destination is selected and its streaming address is shown

#### Scenario: Not connected
- **WHEN** the encoder is not connected
- **THEN** destination selection is unavailable

### Requirement: Device discovery
The device connector SHALL offer scanning the network for devices, indicate that a scan is running, and list what was found with each device's identity and address. A scan finding nothing SHALL say so.

#### Scenario: Scanning
- **WHEN** the user starts a scan
- **THEN** progress is indicated and discovered devices are listed with their identity and address

#### Scenario: Nothing found
- **WHEN** a scan completes with no devices
- **THEN** an empty result is communicated

### Requirement: Operator summary
Connector management SHALL present a summary of the current operating route, covering the selected destination, the encoder's readiness, enabled and ready counts, the streaming target, the device layer's state and the publishing account's state.

#### Scenario: Summary reflects live state
- **WHEN** connector management is open
- **THEN** the summary reflects the current destination, encoder readiness, counts, streaming target, device state and account state

### Requirement: Settings run on live data
Every value on the settings and connector screens SHALL come from the core through the shared client package. Neither screen SHALL contain sample connectors, sample jobs or sample accounts.

#### Scenario: No fixture content
- **WHEN** the settings and connector screens are inspected
- **THEN** no hard-coded connector, job or account is used as a data source

#### Scenario: Failures are reported
- **WHEN** a settings or connector operation fails
- **THEN** the failure is reported and the user can retry
