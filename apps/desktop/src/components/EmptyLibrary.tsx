import { FeatureCard } from './FeatureCard';

const features = [
  {
    title: '本地优先',
    description: '照片、视频、人脸和企业素材默认保存在本机，不做默认上传。'
  },
  {
    title: '自动整理',
    description: '后续自动识别人、地点、事件、产品、截图、废片和重复素材。'
  },
  {
    title: 'AI 修图',
    description: '后续支持基础非破坏式修图、人像精修、产品图精修和批量导出。'
  }
];

export function EmptyLibrary() {
  return (
    <section className="content-area">
      <div className="hero-card">
        <div className="hero-badge">Phase 2 · Desktop Shell</div>
        <h2>还没有导入照片或视频</h2>
        <p>选择一个本地文件夹，影核 AI 将在本机建立你的私人影像库。</p>
        <button className="primary-button large" type="button">导入本地文件夹</button>
      </div>

      <div className="feature-grid">
        {features.map((feature) => (
          <FeatureCard key={feature.title} title={feature.title} description={feature.description} />
        ))}
      </div>
    </section>
  );
}
