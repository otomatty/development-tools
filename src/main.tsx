import { createRoot } from 'react-dom/client';
import App from './App';
import '/styles.css';

const root = document.getElementById('root');

if (!root) {
  throw new Error('Root element not found. Make sure there is a <div id="root"></div> in your index.html');
}

createRoot(root).render(<App />);
