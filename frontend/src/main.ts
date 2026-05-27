import { mount } from 'svelte'
import App from './App.svelte'
import './lib/tokens.css'

// Fix Monaco Editor worker loading
// @ts-ignore
window.MonacoEnvironment = {
  getWorkerUrl: function (_moduleId: any, label: string) {
    if (label === 'json') {
      return '/app-v2/assets/json.worker.js';
    }
    if (label === 'css' || label === 'scss' || label === 'less') {
      return '/app-v2/assets/css.worker.js';
    }
    if (label === 'html' || label === 'handlebars' || label === 'razor') {
      return '/app-v2/assets/html.worker.js';
    }
    if (label === 'typescript' || label === 'javascript') {
      return '/app-v2/assets/ts.worker.js';
    }
    return '/app-v2/assets/editor.worker.js';
  }
};

const app = mount(App, {
  target: document.getElementById('app')!,
})

export default app
