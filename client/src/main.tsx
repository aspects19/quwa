import { StrictMode } from 'react';
import ReactDOM from 'react-dom/client';
import { RouterProvider, createRouter } from '@tanstack/react-router';
import { routeTree } from './routeTree.gen';
import './index.css';

const router = createRouter({ routeTree });

declare module '@tanstack/react-router' {
  interface Register {
    router: typeof router;
  }
}

const rootElement = document.getElementById('root');

if (!rootElement) {
  throw new Error(
    "Root element not found. Ensure that there is an element with id 'root' in your HTML."
  );
}

const root = ReactDOM.createRoot(rootElement);

root.render(
  <StrictMode>
      <RouterProvider router={router} />
  </StrictMode>
);