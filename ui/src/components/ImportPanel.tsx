import { useState } from "react";
import { pickDir, importPhotos } from '../lib/ipc';
import { useImportProgress } from '../hooks/useImportProgress';

interface Props {
    onImportComplete: (libraryDir: string) => void;
}

type ImportMode = 'copyAndImport' | 'importOnly';

export function ImportPanel({ onImportComplete }: Props) {
    const [mode, setMode] = useState<ImportMode>('copyAndImport');
    const [rawOnly, setRawOnly] = useState(false);
    const [source, setSource] = useState<string | null>(null);
    const [dest, setDest] = useState<string | null>(null);
    const [importing, setImporting] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const importedCount = useImportProgress();

    const handlePickSource = async () => {
        const dir = await pickDir();
        if (dir) setSource(dir);
    };

    const handlePickDest = async () => {
        const dir = await pickDir();
        if (dir) setDest(dir);
    }

    const canImport = source !== null && (mode === 'importOnly' || dest !== null);

    const handleImport = async () => {
        if (!source || !canImport) return;
        setImporting(true);
        setError(null);
        try {
            const destArg = mode === 'copyAndImport' ? dest : null;
            await importPhotos(source, destArg, rawOnly);
            onImportComplete(mode === 'copyAndImport' ? dest! : source);
        } catch (e) {
            setError(String(e));
        } finally {
            setImporting(false);
        }
    };

    return (
        <div className="import-panel">
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
            <label className="import-raw-only">
                <input
                    type="checkbox"
                    checked={rawOnly}
                    onChange={(e) => setRawOnly(e.target.checked)}
                    disabled={importing}
                />
                RAW files only
            </label>

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
