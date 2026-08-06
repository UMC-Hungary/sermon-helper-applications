## ADDED Requirements

### Requirement: Current broadcast state
The dashboard SHALL present whether a broadcast is currently live, using a status indicator that animates only while live, a live/off-air label, and elapsed running time. When live it SHALL name the broadcast and its context; when not live it SHALL say so and indicate how a broadcast can be started.

#### Scenario: Live broadcast
- **WHEN** a broadcast is live
- **THEN** the status reads as on air, the indicator animates, the elapsed running time is shown, and the broadcast's name and context are displayed

#### Scenario: No broadcast
- **WHEN** no broadcast is live
- **THEN** the status reads as off air, the indicator does not animate, elapsed time is shown as absent
- **AND** the screen indicates that a broadcast can be scheduled or started

#### Scenario: State changes without reload
- **WHEN** a broadcast starts or stops
- **THEN** the dashboard reflects the change without a manual refresh

### Requirement: Broadcast telemetry
The dashboard SHALL present broadcast telemetry alongside the current state. Each telemetry value SHALL either be sourced from the core or be resolved as a recorded gap; no telemetry value SHALL be displayed as real when it is not.

#### Scenario: Telemetry available
- **WHEN** the core supplies a telemetry value
- **THEN** it is displayed with its unit, and updates as the core reports changes

#### Scenario: Telemetry unavailable
- **WHEN** the core cannot supply a telemetry value the reference displays
- **THEN** the value is omitted or shown as unavailable, according to the recorded decision for it
- **AND** no invented or placeholder figure is presented as real

#### Scenario: No broadcast
- **WHEN** no broadcast is live
- **THEN** telemetry values are shown as absent rather than as zero measurements

### Requirement: Next scheduled event
The dashboard SHALL show the next upcoming event with its date, scheduled time and publishing destinations, and SHALL label the section with when that event occurs.

#### Scenario: Upcoming event exists
- **WHEN** at least one future event is scheduled
- **THEN** the soonest is shown with its date, time and destinations

#### Scenario: No upcoming event
- **WHEN** no future event is scheduled
- **THEN** the section communicates that nothing is scheduled rather than rendering an empty row

### Requirement: Quick actions
The dashboard SHALL offer direct actions for the tasks the reference exposes: creating an event, starting an immediate broadcast, opening presentations, and opening connector management. Each SHALL show a short description, and each SHALL navigate to or perform its action.

#### Scenario: Action navigates
- **WHEN** the user activates a quick action that opens a screen
- **THEN** that screen opens

#### Scenario: Connector action reflects state
- **WHEN** the connectors quick action is shown
- **THEN** it reflects the current number of linked connectors from live data

### Requirement: Dashboard runs on live data
Every value the dashboard displays SHALL come from the core through the shared client package. The dashboard SHALL NOT contain sample events, sample broadcasts or sample telemetry.

#### Scenario: No fixture content
- **WHEN** the dashboard's source is inspected
- **THEN** no hard-coded event, broadcast or telemetry value is used as a data source

#### Scenario: Loading and failure are handled
- **WHEN** dashboard data is loading, or fails to load
- **THEN** the screen presents a loading state or an error state with a way to retry
- **AND** it does not present stale or empty values as current
