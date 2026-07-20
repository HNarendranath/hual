import { useState } from 'react';
import { PhotoGrid } from './components/PhotoGrid';
import { Sidebar } from './components/Sidebar';
import { usePhotos } from './hooks/usePhotos';
import { PhotoFilters, EMPTY_FILTERS } from './lib/ipc';
import './styles/photo-grid.css';
import './styles/filter-panel.css';
import './styles/import-panel.css';
import './styles/sidebar.css';

export default function App() {
    const [libraryDir, setLibraryDir] = useState<string | null>(null);
    const [filters, setFilters] = useState<PhotoFilters>(EMPTY_FILTERS);
    const [sidebarOpen, setSidebarOpen] = useState(true);

    const dbPath = libraryDir ? `${libraryDir}/.hual/hual.db` : null;
    const thumbcacheDir = libraryDir ? `${libraryDir}/.hual/thumbcache` : null;
    const { photos, loading, error, refetch } = usePhotos(dbPath, filters);

    const handleImportComplete = (dir: string) => {
        setLibraryDir(dir);
        refetch();
        setSidebarOpen(false);
    };

    return (
        <div className="app-shell">
            <Sidebar
                open={sidebarOpen}
                onToggle={() => setSidebarOpen((v) => !v)}
                onImportComplete={handleImportComplete}
                filters={filters}
                onFiltersChange={setFilters}
            />
            <PhotoGrid
                photos={photos}
                loading={loading}
                error={error}
                thumbcacheDir={thumbcacheDir}
                sidebarOpen={sidebarOpen}
                onOpenSidebar={() => setSidebarOpen(true)}
            />
        </div>
    );
}
