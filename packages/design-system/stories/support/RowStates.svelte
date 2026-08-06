<script lang="ts">
  import Dot from '../../src/primitives/Dot.svelte';
  import List from '../../src/primitives/List.svelte';
  import Row from '../../src/primitives/Row.svelte';
  import SectionLabel from '../../src/primitives/SectionLabel.svelte';
  import Toggle from '../../src/primitives/Toggle.svelte';
  import ReferencePanel from './ReferencePanel.svelte';

  let notify = $state(true);
</script>

<div class="sanctum-states">
  <figure class="sanctum-state">
    <figcaption>Static, with a leading mark and a trailing control</figcaption>
    <List>
      <Row title="On air" meta="Started 09:31" chevron={false}>
        {#snippet icon()}<Dot color="var(--status-live)" size={7} pulse />{/snippet}
      </Row>
      <Row title="Notify the team" chevron={false} last>
        {#snippet control()}<Toggle bind:checked={notify} label="Notify the team" />{/snippet}
      </Row>
    </List>
  </figure>

  <figure class="sanctum-state">
    <figcaption>Interactive — link, button, current, danger, disabled</figcaption>
    <List>
      <Row title="Service events" meta="12 upcoming" href="#events" />
      <Row title="Dashboard" href="#dashboard" current="page" />
      <Row title="Reconnect the encoder" onclick={() => {}} />
      <Row title="Restart the stream" onclick={() => {}} disabled />
      <Row title="Delete this event" danger onclick={() => {}} last />
    </List>
  </figure>

  <figure class="sanctum-state">
    <figcaption>Under a section label, with a detail column</figcaption>
    <SectionLabel hint="3 sources">Inputs</SectionLabel>
    <List>
      <Row title="Main camera" detail="HDMI 1" chevron={false} />
      <Row title="Pulpit microphone" detail="Rode NT" chevron={false} />
      <Row title="Ambience" detail="Muted" chevron={false} last />
    </List>
  </figure>

  <ReferencePanel component="Row" />
</div>
