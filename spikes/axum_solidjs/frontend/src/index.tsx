import { render } from 'solid-js/web';
import '@picocss/pico/css/pico.min.css';
import App from './App';

const root = document.getElementById('root');
if (!root) {
  throw new Error('missing #root mount point');
}
render(() => <App />, root);
