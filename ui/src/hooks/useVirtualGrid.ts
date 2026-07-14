import { useState, useRef, useLayoutEffect, useCallback } from 'react';

interface Options {
    itemCount: number;
    tileSize: number;
    gap: number;
    overscanRows?: number;
}

interface VirtualGrid {
    containerRef: React.RefObject<HTMLDivElement | null>;
    columns: number;
    startIndex: number;
    endIndex: number;      // exclusive
    totalHeight: number;   // full scrollable height, drives the spacer div
    offsetY: number;       // where the rendered window sits inside that spacer
    direction: 'up' | 'down';
}

export function useVirtualGrid({ itemCount, tileSize, gap, overscanRows = 2 }: Options): VirtualGrid {
    const containerRef = useRef<HTMLDivElement | null>(null);
    const lastScrollTop = useRef(0);

    const [scrollTop, setScrollTop] = useState(0);
    const [containerSize, setContainerSize] = useState({ width: 0, height: 0 });
    const [direction, setDirection] = useState<'up' | 'down'>('down');

    useLayoutEffect(() => {
        const el = containerRef.current;
        if (!el) return;

        const observer = new ResizeObserver(([entry]) => {
            setContainerSize({ width: entry.contentRect.width, height: entry.contentRect. height});
        });
        observer.observe(el);
        return () => observer.disconnect();
    }, []);

    const handleScroll = useCallback(() => {
        const el = containerRef.current;
        if (!el) return;
        const top = el.scrollTop;
        setDirection(top > lastScrollTop.current ? 'down' : 'up');
        lastScrollTop.current = top;
        setScrollTop(top);
    }, []);

    useLayoutEffect(() => {
        const el = containerRef.current;
        if (!el) return;
        el.addEventListener('scroll', handleScroll);
        return () => el.removeEventListener('scroll', handleScroll);
    }, [handleScroll]);

    const columns = Math.max(1, Math.floor(containerSize.width / (tileSize+gap)));
    const rowHeight = tileSize+gap;
    const totalRows = Math.ceil(itemCount/columns);
    const totalHeight = totalRows * rowHeight;

    const startRow = Math.max(0, Math.floor(scrollTop/rowHeight) - overscanRows);
    const visibleRowCount = Math.ceil(containerSize.height / rowHeight) + overscanRows*2;
    const endRow = Math.min(totalRows, startRow + visibleRowCount);
    console.log('containerSize:', containerSize, 'columns:', columns);
    return {
        containerRef,
        columns,
        startIndex: startRow * columns,
        endIndex: Math.min(itemCount, endRow * columns),
        totalHeight,
        offsetY: startRow*rowHeight,
        direction,
    };

}