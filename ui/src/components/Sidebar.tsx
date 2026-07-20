import { PanelLeftClose } from 'lucide-react';
import { ImportPanel } from './ImportPanel';
import { FilterPanel } from './FilterPanel';
import { PhotoFilters } from '../lib/ipc';

interface Props {
    open: boolean;
    onToggle: () => void;
    onImportComplete: (libraryDir: string) => void;
    filters: PhotoFilters;
    onFiltersChange: (filters: PhotoFilters) => void;
}

export function Sidebar({ open, onToggle, onImportComplete, filters, onFiltersChange }: Props) {
    return (
        <div className={`sidebar ${open ? '' : 'collapsed'}`}>
            <div className="sidebar-inner">
                <div className="sidebar-header">
                    <h2>hual</h2>
                    <button className="icon-button" onClick={onToggle} aria-label="Collapse sidebar">
                        <PanelLeftClose size={18} />
                    </button>
                </div>

                <div className="sidebar-section">
                    <h3>Import</h3>
                    <ImportPanel onImportComplete={onImportComplete} />
                </div>

                <div className="sidebar-section">
                    <h3>Filters</h3>
                    <FilterPanel filters={filters} onChange={onFiltersChange} />
                </div>
            </div>
        </div>
    );
}
