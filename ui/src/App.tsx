import { PhotoGrid } from './components/PhotoGrid';
import { useState } from 'react';
import { ImportDialog } from './components/ImportDialog';
import './styles/photo-grid.css';

export default function App() {
    const [libraryDir, setLibraryDir] = useState<string | null>(null);
    
    if (!libraryDir) {
        return <ImportDialog onImportComplete={setLibraryDir} />;
    }

    return (
        <PhotoGrid
            dbPath={`${libraryDir}/.hual/hual.db`}
            thumbcacheDir={`${libraryDir}/.hual/thumbcache`}
        />
    );
}
