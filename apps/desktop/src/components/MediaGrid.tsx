import type { MediaAsset } from '../types';

type MediaGridProps = {
  assets: MediaAsset[];
  totalMediaCount: number;
  photoCount: number;
  videoCount: number;
  searchQuery: string;
  errorText: string | null;
};

function formatFileSize(value: number) {
  if (value < 1024) {
    return `${value} B`;
  }

  const units = ['KB', 'MB', 'GB', 'TB'];
  let size = value / 1024;
  let unitIndex = 0;

  while (size >= 1024 && unitIndex < units.length - 1) {
    size /= 1024;
    unitIndex += 1;
  }

  return `${size.toFixed(size >= 100 ? 0 : 1)} ${units[unitIndex]}`;
}

function formatDate(value: number | null) {
  if (!value) {
    return '未知时间';
  }

  return new Date(value * 1000).toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit'
  });
}

function cameraLabel(asset: MediaAsset) {
  return [asset.camera_make, asset.camera_model].filter(Boolean).join(' ');
}

export function MediaGrid(props: MediaGridProps) {
  const visibleCount = props.assets.length;
  const isSearching = props.searchQuery.trim().length > 0;

  return (
    <section className="media-library">
      <div className="library-header">
        <div>
          <div className="hero-badge">Phase 4 · EXIF Metadata</div>
          <h2>本地媒体库</h2>
          <p>
            已入库 {props.totalMediaCount} 个文件，其中照片 {props.photoCount} 个，视频 {props.videoCount} 个。
          </p>
        </div>

        <div className="library-stat-stack">
          <span>当前显示</span>
          <strong>{visibleCount}</strong>
        </div>
      </div>

      {props.errorText && <div className="library-warning">{props.errorText}</div>}

      {isSearching && visibleCount === 0 ? (
        <div className="empty-result-card">
          <h3>没有找到匹配结果</h3>
          <p>当前阶段支持文件名、扩展名、路径、媒体类型、拍摄年份和相机型号搜索。自然语言搜图将在后续 AI 阶段实现。</p>
        </div>
      ) : (
        <div className="asset-grid">
          {props.assets.map((asset) => (
            <article className="asset-card" key={asset.id} title={asset.file_path}>
              <div className={asset.media_type === 'video' ? 'asset-thumb video' : 'asset-thumb'}>
                <span>{asset.media_type === 'video' ? '▶' : '▧'}</span>
              </div>
              <div className="asset-info">
                <h3>{asset.file_name}</h3>
                <p>{asset.media_type === 'video' ? '视频' : '照片'} · {asset.extension.toUpperCase()} · {formatFileSize(asset.file_size)}</p>
                <p>{asset.taken_at ? `拍摄 ${asset.taken_at}` : `文件 ${formatDate(asset.modified_at ?? asset.created_at)}`}</p>
                {cameraLabel(asset) && <p className="asset-exif">相机 {cameraLabel(asset)}</p>}
              </div>
            </article>
          ))}
        </div>
      )}
    </section>
  );
}
