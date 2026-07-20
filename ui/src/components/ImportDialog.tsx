import { useState } from "react";
import { importPhotos } from '../lib/ipc';
import { useImportProgress } from '../hooks/useImportProgress';

interface Props {
    onImportComplete: (destDir: string) => void;
}

type ImportMode = 'copyAndImport' | 'importOnly';

export function ImportDialog({ onImportComplete }: Props) {
    const [mode, setMode] = useState<ImportMode>('copyAndImport');
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

    const canImport = source !== null && (mode === 'importOnly' || dest !== null);

    const handleImport = async () => {
        if (!source || !canImport) return;
        setImporting(true);
        setError(null);
        try {
            const destArg = mode === 'copyAndImport' ? dest : null;
            await importPhotos(source, destArg);
            onImportComplete(mode === 'copyAndImport' ? dest! : source);
        } catch (e) {
            setError(String(e));
        } finally {
            setImporting(false);
        }
    };

    return (
        <div className="import-dialog">
            <h2>Import Photos</h2>
            <div className="import-mode-toggle">
                <button
                    className={mode === 'copyAndImport' ? 'active' : ''}
                    onClick={() => setMode('copyAndImport')}
                    disabled={importing}
                >
                    Copy & Import
                </button>
                <button
                    className={mode === 'importOnly' ? 'active' : ''}
                    onClick={() => setMode('importOnly')}
                    disabled={importing}
                >
                    Import Only
                </button>
            </div>
            <div className="import-actions">
                <button onClick={handlePickSource} disabled={importing}>
                    {source ?? 'Choose source folder'}
                </button>
                {mode === 'copyAndImport' && (
                    <button onClick={handlePickDest} disabled={importing}>
                        {dest ?? 'Choose destination folder'}
                    </button>
                )}
                <button className="primary" onClick={handleImport} disabled={!canImport || importing}>
                    {importing ? 'Importing...' : 'Import'}
                </button>
            </div>
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