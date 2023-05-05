import React from "react";
import { Theme } from "../base/theme";
import { MathlinguaContext } from "../base/context";

export function useTheme(): Theme {
  const context = React.useContext(MathlinguaContext);
  if (context === null || context === undefined) {
    throw Error('useTheme must be used withing a MathlinguaContext.Provider');
  }
  const theme = context.theme;
  if (theme === undefined) {
    throw Error('the MathlinguaContext.Provider did not specify a theme');
  }
  return theme;
}
