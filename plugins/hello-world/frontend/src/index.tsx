import React from 'react';
import { createRoot } from 'react-dom/client';

interface helloworldProps {
  // Define your component props here
}

const helloworld: React.FC<helloworldProps> = (props) => {
  return (
    <div className="plugin-container">
      <h2>hello-world Plugin</h2>
      <p>Welcome to your custom Judicia plugin!</p>
      
      <div className="plugin-content">
        {/* Add your plugin UI components here */}
        <button 
          onClick={() => console.log('Plugin button clicked!')}
          className="btn btn-primary"
        >
          Plugin Action
        </button>
      </div>
    </div>
  );
};

// Export the component for plugin system
export default helloworld;

// If this is a standalone plugin page, you can render it directly
if (typeof document !== 'undefined') {
  const container = document.getElementById('plugin-root');
  if (container) {
    const root = createRoot(container);
    root.render(<helloworld />);
  }
}
