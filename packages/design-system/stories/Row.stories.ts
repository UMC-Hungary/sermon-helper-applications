import type { Meta, StoryObj } from '@storybook/svelte-vite';
import Row from '../src/primitives/Row.svelte';
import RowStates from './support/RowStates.svelte';

const meta = {
  title: 'Primitives/Row',
  component: Row,
  tags: ['autodocs'],
  parameters: {
    docs: { description: { component: 'See docs/Row.md for the full specification.' } },
  },
  argTypes: {
    current: { control: 'select', options: [false, 'page', 'step'] },
  },
} satisfies Meta<Row>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  args: { title: 'Sunday morning service', meta: 'Every week · 09:30', chevron: false },
};

export const WithDetail: Story = {
  args: { title: 'Stream target', detail: 'YouTube', chevron: false },
};

export const Link: Story = {
  args: { title: 'Service events', meta: '12 upcoming', href: '#events' },
};

export const Button: Story = {
  args: { title: 'Reconnect the encoder', onclick: () => {} },
};

export const Current: Story = {
  args: { title: 'Dashboard', href: '#dashboard', current: 'page' },
};

export const Danger: Story = {
  args: { title: 'Delete this event', danger: true, onclick: () => {} },
};

export const Disabled: Story = {
  args: { title: 'Restart the stream', onclick: () => {}, disabled: true },
};

export const LongTitle: Story = {
  args: {
    title: 'Vasárnapi istentisztelet közvetítése a gyülekezeti nagyteremből, tolmácsolással',
    meta: 'Ez a másodsoros szöveg elég hosszú ahhoz, hogy levágásra kerüljön',
    href: '#long',
  },
};

/** Every state at once, which is how a reviewer compares them against the reference. */
export const AllStates: Story = {
  render: () => ({ Component: RowStates }),
};
