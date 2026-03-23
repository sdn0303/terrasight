"use client";

import { useEffect, useState } from "react";

/**
 * Returns true when the given CSS media query matches.
 * Initialises synchronously on the server to `false` (SSR-safe).
 */
export function useMediaQuery(query: string): boolean {
  // Always initialise to false — matches SSR output and avoids hydration mismatch.
  // The real value is set in useEffect after mount.
  const [matches, setMatches] = useState(false);

  useEffect(() => {
    const mediaQueryList = window.matchMedia(query);
    setMatches(mediaQueryList.matches);

    const listener = (event: MediaQueryListEvent) => {
      setMatches(event.matches);
    };

    mediaQueryList.addEventListener("change", listener);
    return () => {
      mediaQueryList.removeEventListener("change", listener);
    };
  }, [query]);

  return matches;
}
