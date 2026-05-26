import { useEffect, useMemo, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { EmptyLibrary } from './components/EmptyLibrary';
import { MediaGrid } from './components/MediaGrid';
import { Sidebar } from './components/Sidebar';
import { StatusBar } from './components/StatusBar';
import { TopBar } from './components/TopBar';
import type { ImportSummary, LibraryOverview } from './types';

const emptyOverview: LibraryOverview = {
  total_media_count: 0,
  photo_count: 0,
  video_count: 0,
  assets: []
};

export default function App() {
  const [overview, setOverview] = useState<LibraryOverview>(emptyOverview);
  const [statusText, setStatusText] = useState('准备就绪');
  const [errorText, setErrorText] = useState<string | null>(null);
  const [isImporting, setIsImporting] = useState(false);
  const [searchQuery, setSearchQuery] = useState('');

  useEffect(() => {
    void refreshLibraryOverview();
  }, []);

  async function refreshLibraryOverview() {
    try {
      const nextOverview = await invoke<LibraryOverview>('get_library_overview');
      setOverview(nextOverview);
    } catch (error) {
      setErrorText(`读取本地媒体库失败：${String(error)}`);
    }
  }

  async function handleImportFolder() {
    setErrorText(null);

    const selected = await open({
      directory: true,
      multiple: false,
      title: '选择要导入的照片或视频文件夹'
    });

    if (!selected || Array.isArray(selected)) {
      return;
    }

    setIsImporting(true);
    setStatusText('正在扫描本地文件夹...');

    try {
      const summary = await invoke<ImportSummary>('import_media_folder', {
        folderPath: selected
      });
      const nextOverview = await invoke<LibraryOverview>('get_library_overview');
      setOverview(nextOverview);
      setStatusText(`导入完成：新增 ${summary.imported_count} 个，跳过 ${summary.skipped_count} 个`);
    } catch (error) {
      setErrorText(`导入失败：${String(error)}`);
      setStatusText('导入失败');
    } finally {
      setIsImporting(false);
    }
  }

  const filteredAssets = useMemo(() => {
    const query = searchQuery.trim().toLowerCase();
    if (!query) {
      return overview.assets;
    }

    return overview.assets.filter((asset) => {
      return [asset.file_name, asset.extension, asset.media_type, asset.file_path]
        .join(' ')
        .toLowerCase()
        .includes(query);
    });
  }, [overview.assets, searchQuery]);

  return (
    <main className="app-shell">
      <Sidebar />
      <section className="workspace">
        <TopBar
          isImporting={isImporting}
          onImportFolder={handleImportFolder}
          searchQuery={searchQuery}
          onSearchQueryChange={setSearchQuery}
        />

        {overview.total_media_count === 0 ? (
          <EmptyLibrary errorText={errorText} isImporting={isImporting} onImportFolder={handleImportFolder} />
        ) : (
          <MediaGrid
            assets={filteredAssets}
            totalMediaCount={overview.total_media_count}
            photoCount={overview.photo_count}
            videoCount={overview.video_count}
            searchQuery={searchQuery}
            errorText={errorText}
          />
        )}

        <StatusBar
          statusText={statusText}
          totalMediaCount={overview.total_media_count}
          photoCount={overview.photo_count}
          videoCount={overview.video_count}
          aiStatus="未开始"
        />
      </section>
    </main>
  );
}
