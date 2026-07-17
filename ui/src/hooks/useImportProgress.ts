import { useState, useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';
import { ImportProgress } from '../lib/ipc';

export function useImportProgress() {
    const [count, setCount] = useState(0);

    useEffect(() => {
        const unlisten = listen<ImportProgress>('import_progress', (event) => {
            setCount(event.payload.count);
        });

        return () => {
            unlisten.then((f) => f());
        };
    }, []);

    return count;
}