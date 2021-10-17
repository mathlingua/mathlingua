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
	"path/filepath"
	"runtime"
	"strings"
)

const MATHLINGUA_VERSION = "0.11.0"

type Release struct {
	mlgUrl string
	jarUrl string
}

func getNewMlgPath() (string, error) {
	cwd, err := os.Getwd()
	if err != nil {
		return "", err
	}
	return path.Join(cwd, ".bin", "mlg.new"), nil
}

func runMathLingua(args []string) (int, error) {
	cmdArgs := []string{
		"-jar",
		path.Join(".bin", "mathlingua.jar"),
	}
	return runCommand("java", cmdArgs)
}

func runCommand(program string, args []string) (int, error) {
	cmd := exec.Command(program, args...)

	stdout, err := cmd.StdoutPipe()
	if err != nil {
		return 1, err
	}

	stderr, err := cmd.StderrPipe()
	if err != nil {
		return 1, err
	}

	err = cmd.Start()
	if err != nil {
		return 1, err
	}

	combined := io.MultiReader(stdout, stderr)
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

func getLatestVersion() (string, error) {
	allVersions, err := getAllVersions()
	if err != nil {
		return "", err
	}

	if len(allVersions) == 0 {
		return "", errors.New("No releases exist")
	}

	return allVersions[0], nil
}

// version is either "latest" or of the form "x.x.x"
func getRelease(version string) (Release, error) {
	if version == "latest" {
		latestVersion, err := getLatestVersion()
		if err != nil {
			return Release{}, err
		}
		version = latestVersion
	}

	ext := ""
	if runtime.GOOS == "windows" {
		ext = ".exe"
	}

	urlPrefix := "https://github.com/DominicKramer/mathlingua/releases/download/"
	return Release{
		mlgUrl: urlPrefix + "v" + version + "/mlg-" + version + "-" + runtime.GOOS + "-" + runtime.GOARCH + ext,
		jarUrl: urlPrefix + "v" + version + "/mathlingua-" + version + ".jar",
	}, nil
}

func getAllVersions() ([]string, error) {
	allUrls, err := getAllUrls()
	if err != nil {
		return nil, err
	}

	result := []string{}
	for _, url := range allUrls {
		withoutPrefix := strings.ReplaceAll(url,
			"https://github.com/DominicKramer/mathlingua/releases/download/v", "")

		index := strings.Index(withoutPrefix, "/")
		version := ""
		if index < 0 {
			version = withoutPrefix
		} else {
			version = withoutPrefix[:index]
		}

		if len(result) == 0 || result[len(result)-1] != version {
			result = append(result, version)
		}
	}

	return result, nil
}

func getAllUrls() ([]string, error) {
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

		assets, ok := item["assets"].([]interface{})
		if !ok {
			return nil, castErr
		}

		if len(assets) == 0 {
			return nil, castErr
		}

		for _, asset := range assets {
			resultMap, ok := asset.(map[string]interface{})
			if !ok {
				return nil, castErr
			}

			browserDownloadUrl := resultMap["browser_download_url"].(string)
			if !ok {
				return nil, castErr
			}

			urls = append(urls, browserDownloadUrl)
		}
	}

	return urls, nil
}

func downloadUrl(url string, to string) error {
	outPath := to + ".part"
	outputFile, err := os.Create(outPath)
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
	if err != nil {
		return err
	}

	return os.Rename(outPath, to)
}

func ensureMathLinguaJarExists(programName string, programArgs []string,
	version string, isUpdating bool) (bool, error) {
	binDir := ".bin"
	_, err := os.Stat(binDir)
	if os.IsNotExist(err) {
		err := os.MkdirAll(binDir, 0755)
		if err != nil {
			return false, err
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

		release, err := getRelease(version)
		if err != nil {
			return false, err
		}

		if release.jarUrl == "" {
			if version == "latest" {
				return false, errors.New("ERROR: Unable to find the latest version of MathLingua")
			} else {
				return false, errors.New("ERROR: Unable to find version " + version + " of MathLingua")
			}
		}

		if release.mlgUrl == "" {
			if version == "latest" {
				return false, errors.New("ERROR: Unable to find the latest version of mlg")
			} else {
				return false, errors.New("ERROR: Unable to find version " + version + " of mlg")
			}
		}

		// download the mathlingua.jar file
		downloadUrl(release.jarUrl, path.Join(".bin", "mathlingua.jar"))

		newMlgPath, err := getNewMlgPath()
		if err != nil {
			return false, err
		}

		downloadUrl(release.mlgUrl, newMlgPath)
		os.Chmod(newMlgPath, 0755)

		_, err = runCommand(newMlgPath, programArgs)
		return false, err
	}

	return true, nil
}

func copy(src, dest string) error {
	from, err := os.Open(src)
	if err != nil {
		return err
	}
	defer from.Close()

	to, err := os.Create(dest)
	if err != nil {
		return err
	}
	defer to.Close()

	_, err = io.Copy(to, from)
	return err
}

func main() {
	programName := filepath.Base(os.Args[0])

	newMlgPath, err := getNewMlgPath()
	if err != nil {
		fmt.Println(err)
		os.Exit(1)
	}

	if programName == "mlg.new" {
		copy(newMlgPath, "mlg")
	} else {
		_, err := os.Stat(newMlgPath)
		if err == nil {
			os.Remove(newMlgPath)
		}
	}

	args := os.Args[1:]
	// when mlg is initially run, it will download version MATHLINGUA_VERSION
	version := MATHLINGUA_VERSION
	isUpdating := false
	if len(args) >= 1 {
		if args[0] == "update" {
			isUpdating = true

			// remove the .bin directory so that the specified
			// version or the latest version is downloaded
			os.RemoveAll(".bin")

			if len(args) >= 2 {
				// use the version given if specified
				if strings.HasPrefix(args[1], "v") {
					version = args[1][1:]
				} else {
					version = args[1]
				}
			} else {
				// otherwise if an explicit version isn't given, update to the latest
				version = "latest"
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

	programArgs := args
	if isUpdating {
		programArgs = []string{"version"}
	}

	runJarFile, err := ensureMathLinguaJarExists(programName, programArgs, version, isUpdating)
	if err != nil {
		fmt.Println(err)
		os.Exit(1)
	}

	if !runJarFile {
		os.Exit(0)
	}

	exitCode, err := runMathLingua(programArgs)
	if err != nil {
		fmt.Println(err)
	}
	os.Exit(exitCode)
}
