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

/** Blackmagic camera. A blank `fingerprint` means trust-on-first-use. */
export const BlackmagicCameraConfigSchema = z.object({
  enabled: z.boolean(),
  host: z.string(),
  fingerprint: z.string(),
  username: z.string(),
  password: z.string(),
  passwordSet: secretSet,
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
  'blackmagic-camera': BlackmagicCameraConfigSchema,
  youtube: YouTubeConfigSchema,
  facebook: FacebookConfigSchema,
  discord: DiscordConfigSchema,
  szentiras: SzentirasConfigSchema,
} as const;

export type ConnectorName = keyof typeof ConnectorConfigSchemas;
export type ConnectorConfigMap = {
  [K in ConnectorName]: z.infer<(typeof ConnectorConfigSchemas)[K]>;
};

/** One camera returned by an mDNS scan. `host` is what the config field takes. */
export const DiscoveredCameraSchema = z.object({
  host: z.string(),
  hostname: z.string(),
  addresses: z.array(z.string()),
  port: z.number().int(),
  deviceName: z.string(),
  productName: z.string(),
  uniqueId: z.string(),
  softwareVersion: z.string(),
});

export const DiscoveredCamerasSchema = z.object({ cameras: z.array(DiscoveredCameraSchema) });

export type DiscoveredCamera = z.infer<typeof DiscoveredCameraSchema>;

/** Where the camera's livestream now points, after YouTube settings were pushed. */
export const CameraStreamTargetSchema = z.object({
  rtmpUrl: z.string(),
  platform: z.string(),
  server: z.string(),
  quality: z.string(),
  url: z.string().nullable(),
});

export type CameraStreamTarget = z.infer<typeof CameraStreamTargetSchema>;

/** The camera's own payloads, forwarded by the core for the camera control screen. */
const ResolutionSchema = z.object({ width: z.number(), height: z.number() });

const ResolutionDescriptorSchema = z.object({
  group: z.string(),
  aspectRatio: z.string(),
  description: z.string(),
  sensorArea: z.string().default(''),
});

export const CameraFormatSchema = z.object({
  codec: z.string(),
  frameRate: z.string(),
  recordResolution: ResolutionSchema,
  sensorResolution: ResolutionSchema,
  resolutionDescriptor: ResolutionDescriptorSchema,
  offSpeedEnabled: z.boolean().default(false),
  offSpeedFrameRate: z.number().default(0),
});

const SupportedFormatSchema = z.object({
  codecs: z.array(z.string()),
  frameRates: z.array(z.string()),
  recordResolution: ResolutionSchema,
  sensorResolution: ResolutionSchema,
  resolutionDescriptor: ResolutionDescriptorSchema,
});

/** A card or drive in the camera's working set. Only `deviceName` is guaranteed. */
const MediaDeviceSchema = z.object({
  deviceName: z.string(),
  index: z.number().default(0),
  activeDisk: z.boolean().default(false),
  volume: z.string().default(''),
  clipCount: z.number().default(0),
  remainingRecordTime: z.number().default(0),
  remainingSpace: z.number().default(0),
  totalSpace: z.number().default(0),
});

/** Where the livestream points now: the platform entry plus the stream key. */
export const CameraPlatformSchema = z.object({
  platform: z.string(),
  server: z.string(),
  quality: z.string(),
  key: z.string().nullable().default(null),
  passphrase: z.string().nullable().default(null),
  url: z.string().nullable().default(null),
});

const PlatformProfileSchema = z.object({
  profile: z.string(),
  lowLatency: z.boolean().default(false),
  configs: z
    .array(
      z.object({
        resolution: z.string(),
        fps: z.string(),
        bitrate: z.number(),
        audioBitrate: z.number().default(0),
        keyFrameInterval: z.number().default(0),
        videoCodecs: z.array(z.string()).default([]),
      }),
    )
    .default([]),
});

const PlatformServiceSchema = z.object({
  platform: z.string(),
  servers: z.array(z.object({ server: z.string(), url: z.string(), group: z.string().default('') })),
  profiles: z.array(PlatformProfileSchema),
  defaultProfile: z.string().nullable().default(null),
  customizableUrlEnabled: z.boolean().default(false),
});

export const CameraSettingsSchema = z.object({
  recording: z.boolean(),
  record: z.object({
    format: CameraFormatSchema,
    supported: z.object({ supportedFormats: z.array(SupportedFormatSchema) }),
  }),
  storage: z.object({
    slots: z.array(z.object({ index: z.number(), type: z.string() })),
    workingset: z.object({ size: z.number(), workingset: z.array(MediaDeviceSchema.nullable()) }),
    active: z.object({ workingsetIndex: z.number(), deviceName: z.string() }).nullable(),
  }),
  stream: z.object({
    status: z.object({
      status: z.string(),
      bitrate: z.number().default(0),
      effectiveVideoFormat: z.string().default(''),
      duration: z.number().default(0),
      cache: z.number().default(0),
    }),
    available: z.object({ available: z.boolean(), reasons: z.array(z.string()) }),
    platforms: z.array(z.string()),
    active: CameraPlatformSchema,
    platform: PlatformServiceSchema,
  }),
});

/**
 * What the control screen writes back. The camera validates a record format as a
 * whole, so all four record fields travel together or not at all.
 */
export const CameraSettingsUpdateSchema = z.object({
  record: z
    .object({
      codec: z.string(),
      frameRate: z.string(),
      recordResolution: ResolutionSchema,
      sensorResolution: ResolutionSchema,
    })
    .optional(),
  stream: CameraPlatformSchema.optional(),
});

export type CameraSettings = z.infer<typeof CameraSettingsSchema>;
export type CameraFormat = z.infer<typeof CameraFormatSchema>;
export type CameraSupportedFormat = z.infer<typeof SupportedFormatSchema>;
export type CameraMediaDevice = z.infer<typeof MediaDeviceSchema>;
export type CameraPlatform = z.infer<typeof CameraPlatformSchema>;
export type CameraPlatformProfile = z.infer<typeof PlatformProfileSchema>;
export type CameraSettingsUpdate = z.infer<typeof CameraSettingsUpdateSchema>;

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
