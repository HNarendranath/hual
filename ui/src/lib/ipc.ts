import { invoke } from '@tauri-apps/api/core';

export interface Photo {
    srcPath: string;
    destPath: string;
    exposureTime: number | null;
    fStop: number | null;
    focalLength: number | null;
    iso: number | null;
}

export interface RangeFilter {
    min: number | null;
    max: number | null;
}

export interface PhotoFilters {
    iso: RangeFilter;
    fStop: RangeFilter;
    exposureTime: RangeFilter;
    focalLength: RangeFilter;
}

export interface ImportProgress {
    count: number;
}

export const EMPTY_FILTERS: PhotoFilters = {
    iso: { min: null, max: null },
    fStop: { min: null, max: null },
    exposureTime: { min: null, max: null },
    focalLength: { min: null, max: null },
};

export async function listPhotos(dbPath: string, filters: PhotoFilters): Promise<Photo[]> {
    return await invoke<Photo[]>('list_photos', { dbPath, filters });
}

export async function pickDir() : Promise<string | null>  {
    return await invoke<string | null>('pick_dir');
}

export async function getCachedThumbnail(cacheDir: string, src: string): Promise<Uint8Array<ArrayBuffer> | null> {
    const bytes = await invoke<number[] | null>('get_webp', { cacheDir, src });
    return bytes ? new Uint8Array(bytes): null;
}

export async function getFullThumbnail(path: string): Promise<Uint8Array<ArrayBuffer>> {
    const bytes = await invoke<number[]>('get_preview', { path });
    return new Uint8Array(bytes);
}

export async function importPhotos(src: string, dest: string): Promise<void> {
    await invoke<void>('import_photos', { src, dest });
}