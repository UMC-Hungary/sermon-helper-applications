## ADDED Requirements

### Requirement: Tiered notifications
Notifications SHALL carry a severity tier, and their presentation SHALL reflect it. Tier SHALL be conveyed by more than colour alone.

#### Scenario: Tier reflected in presentation
- **WHEN** a notification is raised at a given tier
- **THEN** its presentation reflects that tier
- **AND** the tier is distinguishable without relying on colour alone

### Requirement: Transient notifications
Newly raised notifications SHALL appear transiently over the application, limited to a small number shown at once, with the most recent first. Each SHALL be dismissable. A notification marked persistent SHALL remain until dismissed or resolved rather than disappearing on its own.

#### Scenario: Notification appears
- **WHEN** a notification is raised
- **THEN** it appears transiently, above the newest existing ones

#### Scenario: Display is capped
- **WHEN** more notifications are active than the display limit
- **THEN** only the most recent are shown transiently, and the rest remain available in the notification centre

#### Scenario: Persistent notification stays
- **WHEN** a persistent notification is raised
- **THEN** it does not dismiss itself

#### Scenario: Dismissal
- **WHEN** the user dismisses a notification
- **THEN** it is removed from the transient display and marked read

### Requirement: Notification content
A notification SHALL identify its source, state a title, and where applicable carry a body describing the detail. Where the detail is technical output it SHALL be presented as such. Where a notification reflects a connection state, that state SHALL be indicated, and a state still in progress SHALL be distinguishable from a settled one.

#### Scenario: Source and title
- **WHEN** a notification is displayed
- **THEN** its source is identified and its title is shown

#### Scenario: Technical detail
- **WHEN** a notification's detail is technical output
- **THEN** it is presented in a form suited to reading it exactly

#### Scenario: In-progress state
- **WHEN** a notification reflects a state still being attempted
- **THEN** that is distinguishable from a settled state

### Requirement: Notification actions
The system SHALL support notifications offering actions that address what they report, one of which may be distinguished as primary. Activating an action SHALL perform it and resolve the notification.

#### Scenario: Action offered and performed
- **WHEN** a notification carries actions
- **THEN** they are presented, with any primary action distinguished
- **AND** activating one performs it and resolves the notification

### Requirement: Remediation detail
The system SHALL support notifications carrying an ordered explanation of what to do about them. Where present, it SHALL be hidden by default and revealed on request, and the disclosure control SHALL state which action it will perform.

#### Scenario: Revealing remediation
- **WHEN** a notification offers remediation detail and the user requests it
- **THEN** the ordered steps are revealed and the control offers to hide them again

#### Scenario: No remediation
- **WHEN** a notification carries no remediation detail
- **THEN** no disclosure control is shown

### Requirement: Grouped notifications
The system SHALL support a single notification representing several affected sources at once. Such a notification SHALL identify each affected source individually while remaining one notification.

#### Scenario: Group is itemised
- **WHEN** a notification represents several affected sources
- **THEN** each is identified within it
- **AND** it remains one notification rather than several

### Requirement: Notification centre
The application SHALL provide a notification centre listing all active notifications with their full content and actions, offering to clear them all, and stating plainly when there are none. Opening it SHALL mark the listed notifications read.

#### Scenario: Listing notifications
- **WHEN** the centre is opened with active notifications
- **THEN** each is listed with its content and actions, and the total is shown

#### Scenario: Nothing active
- **WHEN** the centre is opened with nothing active
- **THEN** it states that plainly rather than showing an empty list

#### Scenario: Clearing
- **WHEN** the user clears all
- **THEN** active notifications are removed and the centre reflects it

#### Scenario: Opening marks read
- **WHEN** the centre is opened
- **THEN** the notifications it lists are marked read

### Requirement: Unread indicator
The shell SHALL show an indicator giving access to the notification centre, carrying the unread count and reflecting the highest severity currently active. With nothing unread it SHALL be unobtrusive but still reach the centre.

#### Scenario: Unread count shown
- **WHEN** unread notifications exist
- **THEN** the indicator shows their count and reflects the highest active severity

#### Scenario: Nothing unread
- **WHEN** nothing is unread
- **THEN** the indicator carries no count but still opens the centre

#### Scenario: Opening the centre
- **WHEN** the user activates the indicator
- **THEN** the notification centre opens

### Requirement: Notifications are driven by core events
Notifications SHALL originate from core events delivered over the shared client package, not from sample content. Connector failures, recoveries, authentication expiry and reconnection attempts SHALL raise notifications carrying the detail needed to act on them.

#### Scenario: Core event raises a notification
- **WHEN** the core reports a condition warranting the user's attention
- **THEN** a notification is raised carrying its source, severity and detail

#### Scenario: Resolution updates the notification
- **WHEN** a reported condition is resolved at the core
- **THEN** the corresponding notification reflects the resolution

#### Scenario: No fixture notifications
- **WHEN** the notification system is inspected
- **THEN** no hard-coded sample notification is used as a source

### Requirement: Notifications are accessible
Notifications SHALL be announced to assistive technology when raised, with urgency matching their severity, and SHALL be reachable and operable by keyboard. A notification SHALL NOT steal focus from the user's current task.

#### Scenario: Announcement
- **WHEN** a notification is raised
- **THEN** it is announced to assistive technology with urgency matching its severity

#### Scenario: Keyboard operable
- **WHEN** a notification carrying actions is displayed
- **THEN** its actions and dismissal are reachable and operable by keyboard

#### Scenario: Focus is not stolen
- **WHEN** a notification appears while the user is working
- **THEN** focus remains where the user placed it
