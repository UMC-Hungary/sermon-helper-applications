import { z } from 'zod';

export const TitleTemplateSchema = z.object({
  template: z.string(),
});

export type TitleTemplate = z.infer<typeof TitleTemplateSchema>;

export const SlideFolderSchema = z.object({
  path: z.string(),
});

export type SlideFolder = z.infer<typeof SlideFolderSchema>;
