import { PhotoGrid } from './components/PhotoGrid';
import './styles/photo-grid.css';

const DB_PATH = 'C:\\Users\\gn\\Downloads\\hualtest\\.hual\\hual.db';
const THUMBCACHE_DIR = 'C:\\Users\\gn\\Downloads\\hualtest\\.hual\\thumbcache';

export default function App() {
    return <PhotoGrid dbPath={DB_PATH} thumbcacheDir={THUMBCACHE_DIR} />;
}
