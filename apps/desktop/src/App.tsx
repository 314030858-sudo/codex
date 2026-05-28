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

function friendlyError(error: unknown) {
  const message = String(error);

  if (message.toLowerCase().includes('permission') || message.includes('not allowed')) {
    return '权限不足：当前安装包没有成功打开系统文件夹选择权限。请下载最新安装包后重试。';
  }

  return message;
}

export default function App() {
  const [overview, setOverview] = useState<LibraryOverview>(emptyOverview);
  const [statusText, setStatusText] = useState('准备就绪');
  const [errorText, setErrorText] = useState<string | null>(null);
  const [noticeText, setNoticeText] = useState<string | null>(null);
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
      setErrorText(`读取本地媒体库失败：${friendlyError(error)}`);
      setStatusText('读取本地媒体库失败');
    }
  }

  async function handleImportFolder() {
    if (isImporting) {
      return;
    }
    setErrorText(null);
    setNoticeText(null);
    setStatusText('请选择要导入的本地照片文件夹');
    let selected: string | string[] | null;
    try {
      selected = await open({
        directory: true,
        multiple: false,
        title: '选择要导入的照片或视频文件夹'
      });
    } catch (error) {
      setErrorText(`打开系统文件夹选择器失败：${friendlyError(error)}`);
      setStatusText('文件夹选择失败');
      return;
    }

    if (!selected) {
      setStatusText('已取消导入');
      return;
    }

    if (Array.isArray(selected)) {
      setErrorText('文件夹选择器返回了多个路径，请重新选择一个文件夹。');
      setStatusText('文件夹选择失败');
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
      const importMessage = `导入完成：新增 ${summary.imported_count} 个，跳过 ${summary.skipped_count} 个，当前总数 ${summary.total_media_count} 个。`;
      setNoticeText(importMessage);
      setStatusText(importMessage);
    } catch (error) {
      setErrorText(`导入失败：${friendlyError(error)}`);
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
      const takenYear = asset.taken_at?.slice(0, 4) ?? '';

      return [
        asset.file_name,
        asset.extension,
        asset.media_type,
        asset.file_path,
        asset.taken_at ?? '',
        takenYear,
        asset.camera_make ?? '',
        asset.camera_model ?? '',
        asset.lens_model ?? ''
      ]
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
          <EmptyLibrary
            errorText={errorText}
            noticeText={noticeText}
            isImporting={isImporting}
            onImportFolder={handleImportFolder}
          />
        ) : (
          <MediaGrid
            assets={filteredAssets}
            totalMediaCount={overview.total_media_count}
            photoCount={overview.photo_count}
            videoCount={overview.video_count}
            searchQuery={searchQuery}
            errorText={errorText}
            noticeText={noticeText}
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
