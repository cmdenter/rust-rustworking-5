import React, { useState, useEffect } from 'react';
import ReactDOM from 'react-dom/client';
import { backend } from 'declarations/backend';
import '/index.css';

const App = () => {
  const [currentPoem, setCurrentPoem] = useState('');
  const [currentTitle, setCurrentTitle] = useState('');
  const [poemCount, setPoemCount] = useState(0);
  const [lastUpdate, setLastUpdate] = useState('');
  const [currentPrompt, setCurrentPrompt] = useState('');
  const [nextPrompt, setNextPrompt] = useState('');
  const [isEvolutionLoading, setIsEvolutionLoading] = useState(false);
  const [evolutionResult, setEvolutionResult] = useState(null);

  // Load today's poem on component mount
  useEffect(() => {
    loadTodaysPoem();
  }, []);

  const loadTodaysPoem = async () => {
    try {
      const currentPoemResult = await backend.get_current_poem();
      const poetStateResult = await backend.get_poet_state();
      
      if (currentPoemResult && currentPoemResult.length > 0) {
        const poemData = currentPoemResult[0];
        setCurrentPoem(poemData.poem);
        setCurrentTitle(poemData.title);
        setNextPrompt(poemData.next_prompt);
      }
      
      if (poetStateResult && poetStateResult.length > 0) {
        const state = poetStateResult[0];
        setPoemCount(Number(state.total_poems));
        setCurrentPrompt(state.genesis_prompt);
        
        // Convert nanoseconds to readable date
        const date = new Date(Number(state.last_updated) / 1000000);
        setLastUpdate(date.toLocaleDateString());
      }
    } catch (e) {
      console.error('Error loading poem:', e);
      setCurrentPoem('UNABLE TO LOAD POEM');
      setCurrentTitle('ERROR');
    }
  };

  const testEvolution = async () => {
    // Prevent multiple simultaneous evolutions
    if (isEvolutionLoading) {
      console.log('Evolution already in progress, skipping...');
      return;
    }

    setIsEvolutionLoading(true);
    setEvolutionResult(null);
    
    try {
      console.log('🚀 Starting evolution process...');
      
      // Call backend evolution
      const result = await backend.evolve_poet();
      console.log('✅ Evolution result received:', result);
      
      if (result.Ok) {
        const poemData = result.Ok;
        console.log('🎉 Evolution successful, updating UI...');
        
        // Update poem and title immediately
        setCurrentPoem(poemData.poem || 'No poem received');
        setCurrentTitle(poemData.title || 'Untitled');
        setNextPrompt(poemData.next_prompt || '');
        
        // Increment count
        setPoemCount(prev => {
          const newCount = prev + 1;
          console.log(`📊 Poem count updated: ${prev} → ${newCount}`);
          return newCount;
        });
        
        // Set success result
        setEvolutionResult({
          success: true,
          poem: poemData.poem,
          title: poemData.title,
          next_prompt: poemData.next_prompt
        });
        
        // Refresh all backend state to ensure consistency
        console.log('🔄 Refreshing backend state...');
        await refreshBackendState();
        
      } else {
        console.warn('⚠️ Evolution failed:', result.Err);
        setEvolutionResult({
          success: false,
          poem: `Evolution failed: ${result.Err}`,
          title: 'Evolution Error',
          next_prompt: 'Try again - evolution failed'
        });
        
        setCurrentPoem(`Evolution failed: ${result.Err}\n\nTry clicking evolution again.`);
        setCurrentTitle('Evolution Error');
      }
      
    } catch (e) {
      console.error('💥 Critical error during evolution:', e);
      
      // Set safe error state without breaking the app
      setEvolutionResult({
        poem: `Evolution failed: ${e.message || 'Network or backend error'}`,
        title: 'System Error',
        next_prompt: 'Try again - system error occurred',
        success: false
      });
      
      setCurrentPoem(`System Error: ${e.message || 'Unknown error'}\n\nTry clicking evolution again.`);
      setCurrentTitle('System Error');
      
    } finally {
      setIsEvolutionLoading(false);
      console.log('🏁 Evolution process completed');
    }
  };

  // Helper function to refresh all backend state
  const refreshBackendState = async () => {
    try {
      const poetStateResult = await backend.get_poet_state();
      
      if (poetStateResult && poetStateResult.length > 0) {
        const state = poetStateResult[0];
        setCurrentPrompt(state.genesis_prompt);
        setPoemCount(Number(state.total_poems));
      }
      
      console.log('✅ Backend state refreshed successfully');
    } catch (e) {
      console.warn('⚠️ Could not refresh backend state:', e);
      // Don't throw - this is not critical
    }
  };

  const formatPoem = (poem) => {
    try {
      if (!poem || typeof poem !== 'string') {
        return [(
          <div key="error" className="handwritten-line">
            No poem data available
          </div>
        )];
      }
      
      const poemLines = poem.split('\n');
      const extraBlankLines = 5; // Always add 5 extra blank lines
      
      // Create array with poem lines plus extra blank lines
      const allLines = [...poemLines, ...Array(extraBlankLines).fill('')];
      
      return allLines.map((line, index) => (
        <div key={index} className="handwritten-line">
          {line || '\u00A0'} {/* Non-breaking space for empty lines */}
        </div>
      ));
    } catch (e) {
      console.error('Error formatting poem:', e);
      return [(
        <div key="format-error" className="handwritten-line">
          Error displaying poem
        </div>
      )];
    }
  };

  // Calculate the number of lines needed for the poem plus exactly 5 extra lines (with minimum)
  const calculatePaperHeight = (poem, title) => {
    try {
      const poemLines = (poem && typeof poem === 'string') ? poem.split('\n').length : 1;
      const titleLines = 1; // Title takes 1 line
      const dateLines = 1; // Date takes 1 line
      const extraLines = 5; // ALWAYS exactly 5 extra blank lines at the end
      const emptyLineSpacing = 1; // 1 empty line after date
      
      // Total content lines = actual content + exactly 5 extra lines
      const contentLines = titleLines + dateLines + emptyLineSpacing + poemLines + extraLines;
      const minLines = 24; // MINIMUM 24 lines total (as before)
      const totalLines = Math.max(minLines, contentLines);
      
      // Each line is 32px high, plus top padding
      const lineHeight = 32;
      const topPadding = 32;
      const calculatedHeight = totalLines * lineHeight + topPadding;
      
      return calculatedHeight;
    } catch (e) {
      console.error('Error calculating paper height:', e);
      return 24 * 32 + 32; // Fallback to minimum 24 lines
    }
  };

  return (
    <div className="min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100 p-4 md:p-8 overflow-auto">
      <div className="mx-auto" style={{minWidth: '800px'}}>
        {/* Header */}
        <div className="text-center mb-8">
          <h1 className="text-2xl md:text-4xl mb-4" style={{fontFamily: 'Press Start 2P, monospace'}}>
            SELF-EVOLVING POET
          </h1>
        </div>

        {/* Notebook Container */}
        <div className="flex justify-center">
          <div 
            className="bg-white notebook-paper shadow-2xl relative"
            style={{
              width: '210mm', // About 794px
              minWidth: '794px', // Fixed A4 width in pixels
              minHeight: `${24 * 32 + 32}px`, // Minimum 24 lines + top padding
              border: '1px solid #ddd',
              borderRadius: '4px',
              boxShadow: '0 8px 32px rgba(0,0,0,0.1)',
              flexShrink: 0 // Prevent shrinking
            }}
          >
            {/* Spiral binding holes */}
            <div className="absolute left-6 top-0 bottom-0 flex flex-col justify-around py-4">
              {[...Array(24)].map((_, i) => (
                <div 
                  key={i}
                  className="w-3 h-3 rounded-full bg-gray-100 border border-gray-300"
                />
              ))}
            </div>

            {/* Content Area */}
            <div className="h-full" style={{paddingTop: '32px'}}>
              {/* Title Line - Line 1 */}
              <div className="handwritten-title">
                {currentTitle.toUpperCase() || 'LOADING...'}
              </div>
              
              {/* Date Line - Line 2 */}
              <div className="handwritten-line">
                {new Date().toLocaleDateString('en-US', { 
                  weekday: 'long', 
                  year: 'numeric', 
                  month: 'long', 
                  day: 'numeric' 
                }).toUpperCase()}
              </div>
              
              {/* Empty Line - Line 3 */}
              <div className="handwritten-line"></div>

              {/* Poem Content - Starting from Line 4 */}
              <div>
                {formatPoem(currentPoem.toUpperCase() || 'LOADING POEM...')}
              </div>
            </div>
          </div>
        </div>

        {/* Evolution Controls */}
        <div className="text-center mt-8">
          <button 
            onClick={testEvolution}
            disabled={isEvolutionLoading}
            className="notebook-button px-6 py-3"
          >
            {isEvolutionLoading ? 'EVOLVING...' : 'EVOLVE POET'}
          </button>
        </div>

        {/* Evolution Info */}
        <div className="mt-8 max-w-4xl mx-auto">
          <div className="bg-black text-green-400 p-4 rounded font-mono text-xs">
            <div className="mb-2">
              <span className="text-yellow-400">CURRENT_PROMPT:</span> {currentPrompt || 'Loading...'}
            </div>
            {nextPrompt && (
              <div className="mb-2">
                <span className="text-cyan-400">NEXT_PROMPT:</span> {nextPrompt}
              </div>
            )}
            <div className="mb-2">
              <span className="text-pink-400">EVOLUTION_COUNT:</span> {poemCount}
            </div>
            {evolutionResult && (
              <div className="mt-4 border-t border-gray-600 pt-2">
                <span className="text-red-400">LAST_EVOLUTION_STATUS:</span> {evolutionResult.success ? 'SUCCESS' : 'FAILED'}
              </div>
            )}
          </div>
        </div>

        {/* Bottom Info */}
        <div className="text-center mt-8 text-xs text-gray-500" style={{fontFamily: 'Press Start 2P, monospace'}}>
          INFINITELY EVOLVING AI POET - POWERED BY INTERNET COMPUTER
        </div>
      </div>
    </div>
  );
};

export default App;

ReactDOM.createRoot(document.getElementById('root')).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
