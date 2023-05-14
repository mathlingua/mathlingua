import type { Meta, StoryObj } from '@storybook/react';

import { Button } from '../design/Button';

const meta = {
  title: 'Example/Button',
  component: Button,
  tags: ['autodocs'],
} satisfies Meta<typeof Button>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Primary: Story = {
  args: {
    children: <span>Some text</span>,
  },
};

export const Flat: Story = {
  args: {
    flat: true,
    children: <span>Some text</span>,
  },
};

export const WithAnAriaLabel: Story = {
  args: {
    ariaLabel: 'some-label',
    children: <span>Some text</span>,
  },
};
