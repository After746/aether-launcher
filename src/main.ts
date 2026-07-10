import { mount } from 'svelte';
import App from './App.svelte';
import './lib/styles/tokens.css';
import './lib/styles/global.css';
import { theme } from './lib/stores/theme.svelte';

theme.init();

const app = mount(App, { target: document.getElementById('app')! });

export default app;