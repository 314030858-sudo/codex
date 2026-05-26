const navItems = [
  '全部照片',
  '全部视频',
  '智能相册',
  'AI 修图',
  '批量处理',
  '设置'
];

export function Sidebar() {
  return (
    <aside className="sidebar">
      <div className="brand-block">
        <div className="brand-mark">影</div>
        <div>
          <h1>影核 AI</h1>
          <p>本地影像大脑</p>
        </div>
      </div>

      <nav className="nav-list" aria-label="主导航">
        {navItems.map((item, index) => (
          <button className={index === 0 ? 'nav-item active' : 'nav-item'} key={item} type="button">
            <span className="nav-dot" />
            {item}
          </button>
        ))}
      </nav>

      <div className="sidebar-note">
        <span>本地优先</span>
        <p>照片、视频和人脸数据默认留在本机。</p>
      </div>
    </aside>
  );
}
