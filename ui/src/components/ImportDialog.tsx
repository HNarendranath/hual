import { useState } from "react";
import { pickDir, importPhotos } from '../lib/ipc';
import { useImportProgress } from '../hooks/useImportProgress';

interface Props {
    onImportComplete: (destDir: string) => void;
}

export function ImportDialog({ onImportComplete }: Props) {
    const [source, setSource] = useState<string | null>(null);
    const [dest, setDest] = useState<string | null>(null);
    const [importing, setImporting] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const importedCount = useImportProgress();

    const handlePickSource = async () => {
        // const dir = await pickDir();
        const dir = "C:\\Users\\gn\\Pictures\\hualtesting";
        if (dir) setSource(dir);
    };

    const handlePickDest = async () => {
        // const dir = await pickDir();
        const dir = "C:\\Users\\gn\\Downloads\\hualtest2";
        if (dir) setDest(dir);
    }

    const handleImport = async () => {
        if (!source || !dest) return;
        setImporting(true);
        setError(null);
        try {
            await importPhotos(source, dest);
            onImportComplete(dest);
        } catch (e) {
            setError(String(e));
        } finally {
            setImporting(false);
        }
    };

    return (
        <div className="import-dialog">
            <h2>Import Photos</h2>
            <button onClick={handlePickSource} disabled={importing}>
                {source ?? 'Choose source folder'}
            </button>
            <button onClick={handlePickDest} disabled={importing}>
                {dest ?? 'Choose destination folder'}
            </button>
            <button onClick={handleImport} disabled={!source || !dest || importing}>
                {importing ? 'Importing...' : 'Import'}
            </button>
            {importing && (
                <div className="import-progress">
                    <div className="progress-bar" />
                    <p>Imported {importedCount} photos</p>
                </div>
            )}
            {error && <p className="error">{error}</p>}
        </div>
    );
}