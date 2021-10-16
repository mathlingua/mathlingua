package main

import (
	"bufio"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"net/http"
	"os"
	"os/exec"
	"path"
	"strings"
)

func runMathLingua(args []string) (int, error) {
	cmdArgs := []string{
		"-jar",
		path.Join(".bin", "mathlingua.jar"),
	}
	cmdArgs = append(cmdArgs, args...)
	cmd := exec.Command("java", cmdArgs...)

	stdout, err := cmd.StdoutPipe()
	if err != nil {
		return 1, err
	}

	stderr, err := cmd.StderrPipe()
	if err != nil {
		return 1, err
	}

	combined := io.MultiReader(stdout, stderr)
	err = cmd.Start()
	if err != nil {
		return 1, err
	}

	scanner := bufio.NewScanner(combined)
	for scanner.Scan() {
		fmt.Println(scanner.Text())
	}

	err = cmd.Wait()
	if err != nil {
		return 1, err
	}

	return cmd.ProcessState.ExitCode(), nil
}

func getReleaseUrl(release string) (string, error) {
	allUrls, err := getAllReleaseUrls()
	if err != nil {
		return "", err
	}

	for _, url := range allUrls {
		if release == "latest" {
			return url, nil
		} else if strings.Contains(url, "v"+release) {
			return url, nil
		}
	}

	return "", nil
}

func getAllVersions() ([]string, error) {
	allUrls, err := getAllReleaseUrls()
	if err != nil {
		return nil, err
	}

	result := []string{}
	for _, url := range allUrls {
		withoutPrefix := strings.ReplaceAll(url,
			"https://github.com/DominicKramer/mathlingua/releases/download/v", "")
		index := strings.Index(withoutPrefix, "/")
		if index < 0 {
			result = append(result, withoutPrefix)
		} else {
			result = append(result, withoutPrefix[:index])
		}
	}

	return result, nil
}

func getAllReleaseUrls() ([]string, error) {
	resp, err := http.Get(
		"https://api.github.com/repos/DominicKramer/mathlingua/releases")
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()

	data, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, err
	}

	var jsonData []map[string]interface{}
	json.Unmarshal(data, &jsonData)

	urls := []string{}
	for _, item := range jsonData {
		castErr := errors.New("Could not determine download url")

		asserts, ok := item["assets"].([]interface{})
		if !ok {
			return nil, castErr
		}

		if len(asserts) == 0 {
			return nil, castErr
		}

		resultMap, ok := asserts[0].(map[string]interface{})
		if !ok {
			return nil, castErr
		}

		browserDownloadUrl := resultMap["browser_download_url"].(string)
		if !ok {
			return nil, castErr
		}

		if strings.HasSuffix(browserDownloadUrl, ".jar") {
			urls = append(urls, browserDownloadUrl)
		}
	}

	return urls, nil
}

func downloadUrl(url string) error {
	outputFile, err := os.Create(path.Join(".bin", "mathlingua.jar"))
	if err != nil {
		return err
	}
	defer outputFile.Close()

	resp, err := http.Get(url)
	if err != nil {
		return err
	}
	defer resp.Body.Close()

	_, err = io.Copy(outputFile, resp.Body)
	return err
}

func ensureMathLinguaJarExists(version string, isUpdating bool) error {
	binDir := ".bin"
	_, err := os.Stat(binDir)
	if os.IsNotExist(err) {
		err := os.MkdirAll(binDir, 0755)
		if err != nil {
			return err
		}
	}

	jarfile := path.Join(binDir, "mathlingua.jar")
	_, err = os.Stat(jarfile)
	if os.IsNotExist(err) {
		if !isUpdating {
			fmt.Println("Initial run of mlg detected...")
			fmt.Println()
		}

		if version == "latest" {
			fmt.Println("Downloading the latest version of MathLingua...")
		} else {
			fmt.Println("Downloading version " + version + " of MathLingua...")
		}

		releaseUrl, err := getReleaseUrl(version)
		if err != nil {
			return err
		}

		if releaseUrl == "" {
			if version == "latest" {
				return errors.New("ERROR: Unable to find the latest version of MathLingua")
			} else {
				return errors.New("ERROR: Unable to find version " + version + " of MathLingua")
			}
		}

		downloadUrl(releaseUrl)
	}

	return nil
}

func main() {
	args := os.Args[1:]
	version := "latest"
	isUpdating := false
	if len(args) >= 1 {
		if args[0] == "update" {
			isUpdating = true

			// remove the mathlingua.jar file so either the specified
			// version or the latest version is downloaded
			os.Remove(path.Join(".bin", "mathlingua.jar"))

			if len(args) >= 2 {
				if strings.HasPrefix(args[1], "v") {
					version = args[1][1:]
				} else {
					version = args[1]
				}
			}
		} else if args[0] == "versions" {
			versions, err := getAllVersions()
			if err != nil {
				fmt.Println(err)
				os.Exit(1)
			}

			for index, version := range versions {
				if index == 0 {
					fmt.Println(version + " (latest)")
				} else {
					fmt.Println(version)
				}
			}

			os.Exit(0)
		}
	}

	err := ensureMathLinguaJarExists(version, isUpdating)
	if err != nil {
		fmt.Println(err)
		os.Exit(1)
	}

	if isUpdating {
		exitCode, err := runMathLingua([]string{"version"})
		if err != nil {
			fmt.Println(err)
		}
		os.Exit(exitCode)
	} else {
		exitCode, err := runMathLingua(args)
		if err != nil {
			fmt.Println(err)
		}
		os.Exit(exitCode)
	}
}
