/* @refresh reload */
import { render } from 'solid-js/web';
import App from './App';
import '/styles.css';

const root = document.getElementById('root');

if (!root) {
  throw new Error('Root element not found. Make sure there is a <div id="root"></div> in your index.html');
}

render(() => <App />, root);
