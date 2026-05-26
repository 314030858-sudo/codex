import { EmptyLibrary } from './components/EmptyLibrary';
import { Sidebar } from './components/Sidebar';
import { StatusBar } from './components/StatusBar';
import { TopBar } from './components/TopBar';

export default function App() {
  return (
    <main className="app-shell">
      <Sidebar />
      <section className="workspace">
        <TopBar />
        <EmptyLibrary />
        <StatusBar />
      </section>
    </main>
  );
}
