export type MediaType = 'photo' | 'video';

export type MediaAsset = {
  id: number;
  file_path: string;
  file_name: string;
  extension: string;
  media_type: MediaType;
  file_size: number;
  created_at: number | null;
  modified_at: number | null;
  imported_at: number;
  taken_at: string | null;
  camera_make: string | null;
  camera_model: string | null;
  lens_model: string | null;
  gps_latitude: number | null;
  gps_longitude: number | null;
};

export type LibraryOverview = {
  total_media_count: number;
  photo_count: number;
  video_count: number;
  assets: MediaAsset[];
};

export type ImportSummary = {
  folder_path: string;
  imported_count: number;
  skipped_count: number;
  total_media_count: number;
  assets: MediaAsset[];
};
