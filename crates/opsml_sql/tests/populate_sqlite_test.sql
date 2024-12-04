-- Populate opsml_data_registry
INSERT INTO opsml_data_registry (uid, date, timestamp, app_env, name, repository, major, minor, patch, pre_tag, build_tag, contact, tags, data_type, runcard_uid, pipelinecard_uid, auditcard_uid, interface_type) VALUES 
('data1', '2023-11-29', 1701264000, 'development', 'Data1', 'repo1', 1, 0, 0, 'alpha', 'build1', 'contact1', 'tag1,tag2', 'type1', 'runcard1', 'pipelinecard1', 'auditcard1', 'typeA'),
('data2', '2023-11-29', 1701264001, 'development', 'Data1', 'repo1', 1, 0, 1, 'beta', 'build2', 'contact2', 'tag3,tag4', 'type2', 'runcard2', 'pipelinecard2', 'auditcard2', 'typeB'),
('data3', '2023-11-29', 1701264002, 'development', 'Data1', 'repo1', 1, 1, 0, 'gamma', 'build3', 'contact3', 'tag5,tag6', 'type3', 'runcard3', 'pipelinecard3', 'auditcard3', 'typeC'),
('data4', '2023-11-29', 1701264003, 'development', 'Data1', 'repo1', 1, 1, 1, 'delta', 'build4', 'contact4', 'tag7,tag8', 'type4', 'runcard4', 'pipelinecard4', 'auditcard4', 'typeD'),
('data5', '2023-11-29', 1701264004, 'development', 'Data1', 'repo1', 2, 0, 0, 'epsilon', 'build5', 'contact5', 'tag9,tag10', 'type5', 'runcard5', 'pipelinecard5', 'auditcard5', 'typeE'),
('data6', '2023-11-29', 1701264005, 'development', 'Data1', 'repo1', 2, 0, 1, 'zeta', 'build6', 'contact6', 'tag11,tag12', 'type6', 'runcard6', 'pipelinecard6', 'auditcard6', 'typeF'),
('data7', '2023-11-29', 1701264006, 'development', 'Data1', 'repo1', 2, 1, 0, 'eta', 'build7', 'contact7', 'tag13,tag14', 'type7', 'runcard7', 'pipelinecard7', 'auditcard7', 'typeG'),
('data8', '2023-11-29', 1701264007, 'development', 'Data1', 'repo1', 2, 1, 1, 'theta', 'build8', 'contact8', 'tag15,tag16', 'type8', 'runcard8', 'pipelinecard8', 'auditcard8', 'typeH'),
('data9', '2023-11-29', 1701264008, 'development', 'Data1', 'repo1', 3, 0, 0, 'iota', 'build9', 'contact9', 'tag17,tag18', 'type9', 'runcard9', 'pipelinecard9', 'auditcard9', 'typeI'),
('data10', '2023-11-29', 1701264009, 'development', 'Data1', 'repo1', 3, 0, 1, 'kappa', 'build10', 'contact10', 'tag19,tag20', 'type10', 'runcard10', 'pipelinecard10', 'auditcard10', 'typeJ');

-- Populate opsml_model_registry
INSERT INTO opsml_model_registry (uid, date, timestamp, app_env, name, repository, major, minor, patch, pre_tag, build_tag, contact, tags, datacard_uid, sample_data_type, model_type, runcard_uid, pipelinecard_uid, auditcard_uid, interface_type, task_type) VALUES 
('model1', '2023-11-29', 1701264000, 'development', 'Model1', 'repo1', 1, 0, 0, 'alpha', 'build1', 'contact1', 'tag1,tag2', 'datacard1', 'sample1', 'type1', 'runcard1', 'pipelinecard1', 'auditcard1', 'typeA', 'task1'),
('model2', '2023-11-29', 1701264001, 'development', 'Model2', 'repo2', 1, 0, 1, 'beta', 'build2', 'contact2', 'tag3,tag4', 'datacard2', 'sample2', 'type2', 'runcard2', 'pipelinecard2', 'auditcard2', 'typeB', 'task2'),
('model3', '2023-11-29', 1701264002, 'development', 'Model3', 'repo3', 1, 1, 0, 'gamma', 'build3', 'contact3', 'tag5,tag6', 'datacard3', 'sample3', 'type3', 'runcard3', 'pipelinecard3', 'auditcard3', 'typeC', 'task3'),
('model4', '2023-11-29', 1701264003, 'development', 'Model4', 'repo4', 1, 1, 1, 'delta', 'build4', 'contact4', 'tag7,tag8', 'datacard4', 'sample4', 'type4', 'runcard4', 'pipelinecard4', 'auditcard4', 'typeD', 'task4'),
('model5', '2023-11-29', 1701264004, 'development', 'Model5', 'repo5', 2, 0, 0, 'epsilon', 'build5', 'contact5', 'tag9,tag10', 'datacard5', 'sample5', 'type5', 'runcard5', 'pipelinecard5', 'auditcard5', 'typeE', 'task5'),
('model6', '2023-11-29', 1701264005, 'development', 'Model6', 'repo6', 2, 0, 1, 'zeta', 'build6', 'contact6', 'tag11,tag12', 'datacard6', 'sample6', 'type6', 'runcard6', 'pipelinecard6', 'auditcard6', 'typeF', 'task6'),
('model7', '2023-11-29', 1701264006, 'development', 'Model7', 'repo7', 2, 1, 0, 'eta', 'build7', 'contact7', 'tag13,tag14', 'datacard7', 'sample7', 'type7', 'runcard7', 'pipelinecard7', 'auditcard7', 'typeG', 'task7'),
('model8', '2023-11-29', 1701264007, 'development', 'Model8', 'repo8', 2, 1, 1, 'theta', 'build8', 'contact8', 'tag15,tag16', 'datacard8', 'sample8', 'type8', 'runcard8', 'pipelinecard8', 'auditcard8', 'typeH', 'task8'),
('model9', '2023-11-29', 1701264008, 'development', 'Model9', 'repo9', 3, 0, 0, 'iota', 'build9', 'contact9', 'tag17,tag18', 'datacard9', 'sample9', 'type9', 'runcard9', 'pipelinecard9', 'auditcard9', 'typeI', 'task9'),
('model10', '2023-11-29', 1701264009, 'development', 'Model10', 'repo10', 3, 0, 1, 'kappa', 'build10', 'contact10', 'tag19,tag20', 'datacard10', 'sample10', 'type10', 'runcard10', 'pipelinecard10', 'auditcard10', 'typeJ', 'task10');

-- Populate opsml_run_registry
INSERT INTO opsml_run_registry (uid, date, timestamp, app_env, name, repository, major, minor, patch, pre_tag, build_tag, contact, tags, datacard_uids, modelcard_uids, pipelinecard_uid, project, artifact_uris, compute_environment) VALUES 
('run1', '2023-11-29', 1701264000, 'development', 'Run1', 'repo1', 1, 0, 0, 'alpha', 'build1', 'contact1', 'tag1,tag2', 'datacard1', 'modelcard1', 'pipelinecard1', 'project1', 'uri1', 'env1'),
('run2', '2023-11-29', 1701264001, 'development', 'Run2', 'repo2', 1, 0, 1, 'beta', 'build2', 'contact2', 'tag3,tag4', 'datacard2', 'modelcard2', 'pipelinecard2', 'project2', 'uri2', 'env2'),
('run3', '2023-11-29', 1701264002, 'development', 'Run3', 'repo3', 1, 1, 0, 'gamma', 'build3', 'contact3', 'tag5,tag6', 'datacard3', 'modelcard3', 'pipelinecard3', 'project3', 'uri3', 'env3'),
('run4', '2023-11-29', 1701264003, 'development', 'Run4', 'repo4', 1, 1, 1, 'delta', 'build4', 'contact4', 'tag7,tag8', 'datacard4', 'modelcard4', 'pipelinecard4', 'project4', 'uri4', 'env4'),
('run5', '2023-11-29', 1701264004, 'development', 'Run5', 'repo5', 2, 0, 0, 'epsilon', 'build5', 'contact5', 'tag9,tag10', 'datacard5', 'modelcard5', 'pipelinecard5', 'project5', 'uri5', 'env5'),
('run6', '2023-11-29', 1701264005, 'development', 'Run6', 'repo6', 2, 0, 1, 'zeta', 'build6', 'contact6', 'tag11,tag12', 'datacard6', 'modelcard6', 'pipelinecard6', 'project6', 'uri6', 'env6'),
('run7', '2023-11-29', 1701264006, 'development', 'Run7', 'repo7', 2, 1, 0, 'eta', 'build7', 'contact7', 'tag13,tag14', 'datacard7', 'modelcard7', 'pipelinecard7', 'project7', 'uri7', 'env7'),
('run8', '2023-11-29', 1701264007, 'development', 'Run8', 'repo8', 2, 1, 1, 'theta', 'build8', 'contact8', 'tag15,tag16', 'datacard8', 'modelcard8', 'pipelinecard8', 'project8', 'uri8', 'env8'),
('run9', '2023-11-29', 1701264008, 'development', 'Run9', 'repo9', 3, 0, 0, 'iota', 'build9', 'contact9', 'tag17,tag18', 'datacard9', 'modelcard9', 'pipelinecard9', 'project9', 'uri9', 'env9'),
('run10', '2023-11-29', 1701264009, 'development', 'Run10', 'repo10', 3, 0, 1, 'kappa', 'build10', 'contact10', 'tag19,tag20', 'datacard10', 'modelcard10', 'pipelinecard10', 'project10', 'uri10', 'env10');

-- Populate opsml_audit_registry
INSERT INTO opsml_audit_registry (uid, date, timestamp, app_env, name, repository, major, minor, patch, pre_tag, build_tag, contact, tags, approved, datacards, modelcards, runcards) VALUES 
('audit1', '2023-11-29', 1701264000, 'development', 'Audit1', 'repo1', 1, 0, 0, 'alpha', 'build1', 'contact1', 'tag1,tag2', 1, 'datacard1', 'modelcard1', 'runcard1'),
('audit2', '2023-11-29', 1701264001, 'development', 'Audit2', 'repo2', 1, 0, 1, 'beta', 'build2', 'contact2', 'tag3,tag4', 0, 'datacard2', 'modelcard2', 'runcard2'),
('audit3', '2023-11-29', 1701264002, 'development', 'Audit3', 'repo3', 1, 1, 0, 'gamma', 'build3', 'contact3', 'tag5,tag6', 1, 'datacard3', 'modelcard3', 'runcard3'),
('audit4', '2023-11-29', 1701264003, 'development', 'Audit4', 'repo4', 1, 1, 1, 'delta', 'build4', 'contact4', 'tag7,tag8', 0, 'datacard4', 'modelcard4', 'runcard4'),
('audit5', '2023-11-29', 1701264004, 'development', 'Audit5', 'repo5', 2, 0, 0, 'epsilon', 'build5', 'contact5', 'tag9,tag10', 1, 'datacard5', 'modelcard5', 'runcard5'),
('audit6', '2023-11-29', 1701264005, 'development', 'Audit6', 'repo6', 2, 0, 1, 'zeta', 'build6', 'contact6', 'tag11,tag12', 0, 'datacard6', 'modelcard6', 'runcard6'),
('audit7', '2023-11-29', 1701264006, 'development', 'Audit7', 'repo7', 2, 1, 0, 'eta', 'build7', 'contact7', 'tag13,tag14', 1, 'datacard7', 'modelcard7', 'runcard7'),
('audit8', '2023-11-29', 1701264007, 'development', 'Audit8', 'repo8', 2, 1, 1, 'theta', 'build8', 'contact8', 'tag15,tag16', 0, 'datacard8', 'modelcard8', 'runcard8'),
('audit9', '2023-11-29', 1701264008, 'development', 'Audit9', 'repo9', 3, 0, 0, 'iota', 'build9', 'contact9', 'tag17,tag18', 1, 'datacard9', 'modelcard9', 'runcard9'),
('audit10', '2023-11-29', 1701264009, 'development', 'Audit10', 'repo10', 3, 0, 1, 'kappa', 'build10', 'contact10', 'tag19,tag20', 0, 'datacard10', 'modelcard10', 'runcard10');