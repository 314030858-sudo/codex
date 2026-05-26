type StatusBarProps = {
  statusText: string;
  totalMediaCount: number;
  photoCount: number;
  videoCount: number;
  aiStatus: string;
};

export function StatusBar(props: StatusBarProps) {
  return (
    <footer className="status-bar">
      <span>当前状态：{props.statusText}</span>
      <span>媒体数量：{props.totalMediaCount} 个文件</span>
      <span>照片：{props.photoCount}</span>
      <span>视频：{props.videoCount}</span>
      <span>AI 状态：{props.aiStatus}</span>
    </footer>
  );
}
