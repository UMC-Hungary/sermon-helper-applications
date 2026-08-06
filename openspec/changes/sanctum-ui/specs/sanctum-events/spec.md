## ADDED Requirements

### Requirement: Event list with filters and search
The events screen SHALL list events with, for each, its date, scheduled time, title and publishing destinations. It SHALL offer the filters the reference defines — upcoming, live, past and drafts — and a search over events. The header SHALL summarise the current period and count.

#### Scenario: Filter narrows the list
- **WHEN** the user selects a filter
- **THEN** only events matching it are listed, and the active filter is indicated

#### Scenario: Search narrows the list
- **WHEN** the user enters a search term
- **THEN** the list narrows to matching events
- **AND** a search returning nothing shows an empty state rather than a blank list

#### Scenario: Live and draft events are marked
- **WHEN** an event is live
- **THEN** it carries an animated live indicator
- **AND** a draft event is marked as a draft

### Requirement: Event selection detail
The events screen SHALL present a detail view for the selected event showing its date, time, destinations, title and status, with an action to open it for editing. Where no event is explicitly selected, it SHALL default to the live event if one exists, otherwise the first listed.

#### Scenario: Selecting an event
- **WHEN** the user selects an event
- **THEN** the detail view shows that event's date, time, destinations, title and status

#### Scenario: Default selection
- **WHEN** the screen loads with no explicit selection
- **THEN** a live event is selected if one exists, otherwise the first listed event

#### Scenario: Opening for editing
- **WHEN** the user opens the selected event
- **THEN** the event editor opens for that event

### Requirement: Event creation and editing
The editor SHALL support creating a new event and editing an existing one, presenting the reference's sections: details, scripture, description, generated title preview, publishing privacy and recording options. It SHALL commit through a persistent action area offering cancel and save, and cancelling SHALL return to the events screen without saving.

#### Scenario: Creating an event
- **WHEN** the user completes the editor for a new event and saves
- **THEN** the event is created through the core and appears in the events list

#### Scenario: Editing an event
- **WHEN** the user opens an existing event
- **THEN** its stored values populate the editor
- **AND** saving updates the event through the core

#### Scenario: Cancelling
- **WHEN** the user cancels
- **THEN** no change is persisted and the events screen is shown

#### Scenario: Save failure
- **WHEN** saving fails
- **THEN** the failure is reported, the entered values are retained, and the user can retry

### Requirement: Event details section
The details section SHALL capture the event title, date, time and speaker. The title SHALL enforce a maximum length, display a live character count, and warn as the limit is approached. Date and time SHALL use native pickers presenting a human-readable formatted value.

#### Scenario: Title length feedback
- **WHEN** the user types a title
- **THEN** the character count updates live
- **AND** it warns as the remaining allowance becomes small
- **AND** input beyond the maximum is prevented

#### Scenario: Date and time entry
- **WHEN** the user sets a date or time
- **THEN** a native picker is used and the chosen value is displayed in readable long form

### Requirement: Scripture references with lookup
The scripture section SHALL accept a primary and a secondary reference, each ranked and labelled as the reference defines. Entering a reference SHALL look up its passage after a short pause in typing, display the retrieved text with its translation, and report an unrecognised reference without discarding what was typed.

#### Scenario: Successful lookup
- **WHEN** the user enters a recognised reference and pauses
- **THEN** the passage text and its translation are displayed

#### Scenario: Lookup is debounced
- **WHEN** the user is still typing
- **THEN** no lookup is issued until typing pauses
- **AND** a superseded lookup does not overwrite the result of a later one

#### Scenario: Unrecognised reference
- **WHEN** an entered reference cannot be resolved
- **THEN** it is reported as unrecognised
- **AND** the entered text is retained

#### Scenario: Cleared reference
- **WHEN** the user clears a reference
- **THEN** its passage display is cleared

### Requirement: Generated title preview
The editor SHALL show a live, read-only preview of the title that will be published, composed from the event date, primary scripture reference, title and speaker. It SHALL indicate which of those contributing fields are currently populated, and SHALL show the composed title's length against the publishing limit.

#### Scenario: Preview updates live
- **WHEN** any contributing field changes
- **THEN** the preview recomposes immediately

#### Scenario: Contributing fields indicated
- **WHEN** the preview is shown
- **THEN** each contributing field is indicated as populated or not

#### Scenario: Length warning
- **WHEN** the composed title approaches the publishing limit
- **THEN** the length indication warns

#### Scenario: Nothing to compose
- **WHEN** no contributing field is populated
- **THEN** the preview explains what is needed rather than showing an empty value

### Requirement: Description
The editor SHALL capture a description intended for the publishing destination, in a multi-line field with a live character count against its limit.

#### Scenario: Description entry
- **WHEN** the user enters a description
- **THEN** the character count updates live against the limit

### Requirement: Publishing privacy
The editor SHALL offer the publishing privacy options the reference defines — public, unlisted and private — as a single-choice control where each option carries a label and an explanation of its effect.

#### Scenario: Selecting privacy
- **WHEN** the user selects a privacy option
- **THEN** that option is indicated as selected and applies on save
- **AND** each option's effect is explained

### Requirement: Recording options
The editor SHALL offer automatic upload of the recording after the event, with an explanation of what it does, and a choice of recording visibility including the reference's deferred option that publishes unlisted first and public later.

#### Scenario: Automatic upload
- **WHEN** the user enables automatic upload
- **THEN** the setting is persisted with the event and its effect is explained

#### Scenario: Recording visibility
- **WHEN** the user chooses a recording visibility
- **THEN** the choice is persisted with the event
- **AND** the deferred unlisted-then-public option is available

### Requirement: Events run on live data
Every event, scripture passage and destination shown SHALL come from the core through the shared client package. The screen SHALL NOT contain sample events or sample passages.

#### Scenario: No fixture content
- **WHEN** the events screens are inspected
- **THEN** no hard-coded event or passage is used as a data source

#### Scenario: Empty, loading and error states
- **WHEN** events are loading, unavailable, or absent
- **THEN** the screen presents a loading, error or empty state accordingly
