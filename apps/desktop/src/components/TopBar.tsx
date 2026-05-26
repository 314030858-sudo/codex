export function TopBar() {
  return (
    <header className="top-bar">
      <div className="search-box">
        <span>⌕</span>
        <input placeholder="搜索照片、视频、人物、地点或产品" aria-label="搜索" />
      </div>

      <div className="top-actions">
        <div className="ai-status">AI 分析：未开始</div>
        <button className="primary-button" type="button">导入文件夹</button>
      </div>
    </header>
  );
}
