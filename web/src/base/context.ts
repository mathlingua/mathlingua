import React from "react";
import { Theme } from "./theme";

export interface MathlinguaContextValue {
  theme?: Theme;
}

export const MathlinguaContext = React.createContext<MathlinguaContextValue>({});
