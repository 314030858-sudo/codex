type TopBarProps = {
  isImporting: boolean;
  searchQuery: string;
  onSearchQueryChange: (value: string) => void;
  onImportFolder: () => void;
};

export function TopBar(props: TopBarProps) {
  return (
    <header className="top-bar">
      <div className="search-box">
        <span>⌕</span>
        <input
          placeholder="搜索照片、视频、人物、地点或产品"
          aria-label="搜索"
          value={props.searchQuery}
          onChange={(event) => props.onSearchQueryChange(event.target.value)}
        />
      </div>

      <div className="top-actions">
        <div className="ai-status">AI 分析：未开始</div>
        <button className="primary-button" type="button" onClick={props.onImportFolder}>
          {props.isImporting ? '正在导入...' : '导入文件夹'}
        </button>
      </div>
    </header>
  );
}
