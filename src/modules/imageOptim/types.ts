export type OptimizeStatus = "optimized" | "alreadyOptimal" | "error";

export type OptimizeResult = {
  path: string;
  beforeBytes: number;
  afterBytes: number;
  savedBytes: number;
  status: OptimizeStatus;
  error: string | null;
};

export type ImageOptimSettings = {
  stripMetadata: boolean;
  effort: number;
  lossyEnabled: boolean;
  jpegMaxQuality: number;
  pngMinQuality: number;
  gifQuality: number;
};

export type OptimizeFileInput = {
  path: string;
  lossy?: boolean | null;
};

export const DEFAULT_IMAGE_OPTIM_SETTINGS: ImageOptimSettings = {
  stripMetadata: true,
  effort: 2,
  lossyEnabled: false,
  jpegMaxQuality: 80,
  pngMinQuality: 80,
  gifQuality: 80,
};
