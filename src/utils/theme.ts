export const useTheme = () => {
  const initTheme = () => {
    document.documentElement.classList.add('dark');
  };

  return {
    initTheme
  };
}; 