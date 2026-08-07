import { setProjectAnnotations } from '@storybook/svelte-vite';
import { beforeAll } from 'vitest';
import preview from './preview';

const project = setProjectAnnotations([preview]);

beforeAll(project.beforeAll);
