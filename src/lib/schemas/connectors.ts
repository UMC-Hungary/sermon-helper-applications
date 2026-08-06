import { z } from 'zod';

/**
 * A credential the core never returns. Reads come back blank with a companion
 * `<field>Set` flag telling the UI whether one is stored; writing a blank value
 * keeps whatever the server already has.
 */
const secretSet = z.boolean().optional();

export const ObsConfigSchema = z.object({
  enabled: z.boolean(),
  host: z.string(),
  port: z.number(),
  password: z.string().nullable(),
  passwordSet: secretSet,
});

export const VmixConfigSchema = z.object({
  enabled: z.boolean(),
  host: z.string(),
  port: z.number(),
});

export const AtemConfigSchema = z.object({
  enabled: z.boolean(),
  host: z.string(),
  port: z.number(),
});

export const BroadlinkConfigSchema = z.object({
  enabled: z.boolean(),
});

export const YouTubeConfigSchema = z.object({
  enabled: z.boolean(),
  clientId: z.string(),
  clientSecret: z.string(),
  clientSecretSet: secretSet,
});

export const FacebookConfigSchema = z.object({
  enabled: z.boolean(),
  appId: z.string(),
  appSecret: z.string(),
  appSecretSet: secretSet,
  pageId: z.string(),
});

export const DiscordConfigSchema = z.object({
  enabled: z.boolean(),
  webhookUrl: z.string(),
  webhookUrlSet: secretSet,
});

export const SzentirasConfigSchema = z.object({
  enabled: z.boolean(),
  apiKey: z.string(),
  apiKeySet: secretSet,
});

/** Every connector whose config the core stores, keyed by its API path segment. */
export const ConnectorConfigSchemas = {
  obs: ObsConfigSchema,
  vmix: VmixConfigSchema,
  atem: AtemConfigSchema,
  broadlink: BroadlinkConfigSchema,
  youtube: YouTubeConfigSchema,
  facebook: FacebookConfigSchema,
  discord: DiscordConfigSchema,
  szentiras: SzentirasConfigSchema,
} as const;

export type ConnectorName = keyof typeof ConnectorConfigSchemas;
export type ConnectorConfigMap = {
  [K in ConnectorName]: z.infer<(typeof ConnectorConfigSchemas)[K]>;
};

export const ConnectorStatusPayloadSchema = z.object({
  type: z.enum(['disconnected', 'connecting', 'connected', 'error']),
  message: z.string().optional(),
});

export const ConnectorStatusesSchema = z.record(
  z.enum(Object.keys(ConnectorConfigSchemas) as [ConnectorName, ...ConnectorName[]]),
  ConnectorStatusPayloadSchema,
);

export const ObsStreamSettingsSchema = z.object({
  serviceType: z.string(),
  server: z.string(),
  key: z.string(),
});

export type ObsStreamSettings = z.infer<typeof ObsStreamSettingsSchema>;
